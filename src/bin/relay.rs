use axum::{
    Router,
    body::{Body, BodyDataStream},
    extract::{Query, State},
    http,
    response::{IntoResponse, Response},
    routing::post,
};
use bytes::Bytes;
use futures_util::{StreamExt, TryStreamExt, stream::BoxStream};

use http_body_util::BodyExt;
use http_proxy::error::ProxyError;
use hyper::{HeaderMap, Method, Request, Uri};
use hyper_util::rt::TokioIo;
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{io::copy_bidirectional, net::TcpStream, sync::oneshot, time::timeout};
use tokio::{
    io::{DuplexStream, duplex},
    sync::Mutex,
};
use tokio_util::io::{ReaderStream, StreamReader};

#[derive(Clone)]
struct AppState {
    sessions: Arc<Mutex<HashMap<String, Pending>>>,
}

struct Pending {
    // the first connection's data stream (A -> ...)
    first_stream: BoxStream<'static, Result<bytes::Bytes, axum::Error>>,
    // send the second connection's stream to the first connection
    notify_first: oneshot::Sender<BoxStream<'static, Result<bytes::Bytes, axum::Error>>>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/relay", post(relay_handler))
        .route("/proxy", post(proxy_handler))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9999").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct RelayQuery {
    #[serde(rename = "X-SESSION-ID")]
    sid: Option<String>,
}

#[derive(Deserialize)]
struct ProxyQuery {
    #[serde(rename = "X-PROXY-URL")]
    proxy: String,
}

#[axum::debug_handler]
async fn relay_handler(
    State(state): State<AppState>,
    Query(query): Query<RelayQuery>,
    headers: HeaderMap,
    body: Body,
) -> impl IntoResponse {
    let sid0 = query.sid.or_else(|| {
        headers
            .get("X-SESSION-ID")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    });
    // 获取 session ID
    let sid = match sid0 {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => {
            return Response::new(Body::from("missing X-SESSION-ID header"));
        }
    };

    let incoming_stream = StreamExt::boxed(body.into_data_stream());

    let maybe_first_stream: Option<BoxStream<'static, Result<bytes::Bytes, axum::Error>>> = {
        let mut map = state.sessions.lock().await;
        if let Some(pending) = map.remove(&sid) {
            // there is a pending first connection: take its stream and its sender
            pending.notify_first.send(incoming_stream);
            Some(pending.first_stream)
        } else {
            // no pending: create oneshot pair and store
            let (tx, rx) = oneshot::channel();
            let pending = Pending {
                first_stream: incoming_stream,
                notify_first: tx,
            };

            map.insert(sid.clone(), pending);
            drop(map);
            // we'll wait on rx after releasing lock
            Some(rx.await.unwrap())
        }
    };
    println!("OOKK");
    Response::new(Body::from_stream(maybe_first_stream.unwrap()))
}

#[axum::debug_handler]
async fn proxy_handler(
    Query(query): Query<ProxyQuery>,
    req: http::Request<axum::body::Body>,
) -> impl IntoResponse {
    fetch_url(query.proxy.parse().unwrap(), req).await.unwrap()
}

async fn fetch_url(
    url: hyper::Uri,
    req: http::Request<axum::body::Body>,
) -> Result<axum::response::Response<axum::body::Body>, Box<dyn std::error::Error>> {
    println!("Url:{}", url.to_string());
    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(80);
    let addr = format!("{}:{}", host, port);
    let stream = TcpStream::connect(addr).await?;
    let io = TokioIo::new(stream);

    let (mut sender, conn) = timeout(
        Duration::from_secs(10),
        hyper::client::conn::http1::handshake(io),
    )
    .await??;
    tokio::task::spawn(async move { if let Err(_) = conn.await {} });

    let mut req_builder = Request::builder().uri(url.path_and_query().unwrap().as_str());
    req_builder = req_builder.method(Method::GET);

    for (k, v) in req.headers().iter() {
        req_builder = req_builder.header(k, v);
    }
    let req0 = req_builder.body(req.into_body())?;

    let mut res = timeout(Duration::from_secs(600), sender.send_request(req0)).await??;

    let (parts, body_stream) = res.into_parts();
    let mut data_stream = body_stream.into_data_stream();

    let s = async_stream::stream! {
        loop {
            match tokio::time::timeout(
                Duration::from_secs(60),
                data_stream.next(),
            ).await {
                Ok(Some(chunk)) => {
                    yield chunk.map_err(|e| Box::new(e) as  Box<dyn std::error::Error + Send + Sync>);
                }
                Ok(None) => break,
                Err(_) => {
                    log::warn!("upstream resp body idle timeout: {:?}", url);
                    yield Err(Box::new(ProxyError::Timeout) as Box<dyn std::error::Error + Send + Sync>);
                    break;
                }
            }
        }
    };
    let body = Body::from_stream(s);
    let axum_resp = axum::response::Response::from_parts(parts, body);

    Ok(axum_resp)
}
