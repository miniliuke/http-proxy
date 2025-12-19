use axum::{
    Router,
    body::{Body, BodyDataStream},
    extract::{State},
    response::{IntoResponse, Response},
    routing::post,
};
use bytes::Bytes;
use futures_util::{StreamExt, TryStreamExt, stream::BoxStream};
use hyper::HeaderMap;
use std::{
    collections::HashMap,
    sync::{Arc},
};
use tokio::{io::{DuplexStream, duplex}, sync::Mutex};
use tokio::{io::copy_bidirectional, sync::oneshot};
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
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9999").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[axum::debug_handler]
async fn relay_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Body,
) -> impl IntoResponse {
    // 获取 session ID
    let sid = match headers.get("X-SESSION-ID").and_then(|v| v.to_str().ok()) {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => {
            return Response::new(Body::from("missing X-SESSION-ID header"));
        }
    };

    let incoming_stream = body.into_data_stream().boxed();

    let maybe_first_stream: Option<BoxStream<'static, Result<bytes::Bytes, axum::Error>>> = {
        let mut map = state.sessions.lock().await;
        if let Some(pending) = map.remove(&sid) {
            // there is a pending first connection: take its stream and its sender
            Some(pending.first_stream)
        } else {
            // no pending: create oneshot pair and store
            let (tx, rx) = oneshot::channel();
            let pending = Pending {
                first_stream: incoming_stream,
                notify_first: tx,
            };

            // map.insert(sid.clone(), pending);
            drop(map);
            // we'll wait on rx after releasing lock
            Some(rx.await.unwrap())
        }
    };

    Response::new(Body::from_stream(maybe_first_stream.unwrap()))
}
