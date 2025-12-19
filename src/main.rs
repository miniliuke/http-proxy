use async_compression::{
    Level,
    tokio::bufread::{GzipDecoder, GzipEncoder, ZstdDecoder, ZstdEncoder},
};
use axum::{
    Router,
    body::{Body, Bytes},
    http::{self, HeaderValue, Request, Response, Uri},
};
use bytes::BytesMut;
use clap::Parser;
use futures_util::{Stream, StreamExt, TryStreamExt};
use http_body_util::BodyExt;
use http_proxy::{config::Config, error::ProxyError};
use hyper::header::{CONTENT_ENCODING, CONTENT_LENGTH, TRANSFER_ENCODING};
use hyper_util::rt::TokioIo;
use log::LevelFilter;
use log4rs::{
    append::{
        console::ConsoleAppender,
        rolling_file::{
            RollingFileAppender,
            policy::compound::{
                CompoundPolicy, roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger,
            },
        },
    },
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};
use mimalloc::MiMalloc;
use serde::Deserialize;
use std::{
    collections::HashMap,
    convert::Infallible,
    io,
    net::SocketAddr,
    ops::RangeTo,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::sync::RwLock;
use tokio::{io::AsyncWriteExt as _, time::timeout};
use tokio::{io::BufReader, net::TcpStream};
use tokio_util::io::{ReaderStream, StreamReader};
use tower::{Layer, Service};
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const CUSTOM_ENCODING_HEADER: &str = "crd-custom-encoding";
const CUSTOM_LENGTH: &str = "crd-custom-length";
const CUSTOM_ENCODING_VALUE: &str = "zstd";

#[derive(Clone)]
struct ProxyService {
    config: Arc<Config>,
    targets: Vec<String>,
}

impl ProxyService {
    fn new(config: Arc<Config>) -> Self {
        let targets = config
            .target
            .split(",")
            .map(|target| target.to_string())
            .collect::<Vec<String>>();

        Self { config, targets }
    }
}

impl Service<Request<Body>> for ProxyService {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn futures_util::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let uri = build_upstream_uri(
            &self.targets[rand::random_range(0..self.targets.len())],
            req.uri(),
        );
        Box::pin(async move {
            match fetch_url(uri.clone(), req).await {
                Ok(resp) => Ok(resp),

                Err(err) => {
                    log::warn!("proxy upstream error: {:?} {}", uri, err);
                    let resp = Response::builder()
                        .status(http::StatusCode::BAD_GATEWAY)
                        .body(Body::from("Bad Gateway"))
                        .unwrap();
                    Ok(resp)
                }
            }
        })
    }
}

fn build_upstream_uri(base: &str, original_uri: &Uri) -> Uri {
    // 解析 base uri
    let base_uri: Uri = base.parse().expect("invalid base uri");

    // 提取 path 和 query
    let path_and_query = original_uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");

    // 组合成新的完整字符串
    let new_uri_str = format!(
        "{}://{}{}",
        base_uri.scheme_str().unwrap_or("http"),
        base_uri.authority().unwrap(),
        path_and_query
    );

    // 解析成 Uri
    new_uri_str.parse().expect("failed to build upstream uri")
}

#[derive(Clone)]
struct ProxyLayer {
    config: Arc<Config>,
}

impl<S> Layer<S> for ProxyLayer {
    type Service = ProxyService;

    fn layer(&self, _inner: S) -> Self::Service {
        ProxyService::new(self.config.clone())
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 256)]
async fn main() {
    init_log4rs_rolling("./log", 200, 3);
    let config = Config::parse();
    let cfg = Arc::new(config.clone());

    let app = Router::new().fallback_service(ProxyService::new(cfg.clone()));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    println!("🚀 Listening on http://{}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

fn compress_body(body: Body) -> Body {
    let stream = TryStreamExt::map_err(body.into_data_stream(), |e| {
        io::Error::new(io::ErrorKind::Other, e)
    });
    let reader = StreamReader::new(stream);
    let buf_reader = BufReader::new(reader);
    let gzip = GzipEncoder::with_quality(buf_reader, Level::Precise(6));
    let compressed_stream = ReaderStream::new(gzip).map_ok(Bytes::from);
    Body::from_stream(compressed_stream)
}

pub fn maybe_compress_body(req_method: &http::Method, body: Body) -> Body {
    match *req_method {
        http::Method::POST | http::Method::PUT | http::Method::PATCH => {
            // 将 Body 转成流，再用 GzipEncoder 压缩
            let stream = TryStreamExt::map_err(body.into_data_stream(), |e| {
                io::Error::new(io::ErrorKind::Other, e)
            });

            let reader = StreamReader::new(stream);
            let buf_reader = BufReader::new(reader);
            let encoder = ZstdEncoder::with_quality(buf_reader, Level::Precise(6));
            // let gzip = GzipEncoder::with_quality(buf_reader, Level::Precise(6));
            let compressed_stream = ReaderStream::new(encoder).map_ok(Bytes::from);

            Body::from_stream(compressed_stream)
        }
        http::Method::GET | http::Method::HEAD | http::Method::OPTIONS => {
            // 不压缩，直接返回原 body
            body
        }
        _ => {
            // 其他方法默认不压缩
            body
        }
    }
}

pub fn maybe_decompress_body(req_method: &http::Method, body: Body) -> Body {
    match *req_method {
        http::Method::POST | http::Method::PUT | http::Method::PATCH => {
            let stream = TryStreamExt::map_err(body.into_data_stream(), |e| {
                io::Error::new(io::ErrorKind::Other, e)
            });
            let reader = StreamReader::new(stream);
            let buf_reader = BufReader::new(reader);
            let decoder = ZstdDecoder::new(buf_reader);
            // let gzip = GzipDecoder::new(buf_reader);
            let decompressed_stream = ReaderStream::new(decoder).map_ok(Bytes::from);
            Body::from_stream(decompressed_stream)
        }
        http::Method::GET | http::Method::HEAD | http::Method::OPTIONS => body,
        _ => body,
    }
}
fn decompress_body(body: Body) -> Body {
    let stream = TryStreamExt::map_err(body.into_data_stream(), |e| {
        io::Error::new(io::ErrorKind::Other, e)
    });
    let reader = StreamReader::new(stream);
    let buf_reader = BufReader::new(reader);
    let gzip = GzipDecoder::new(buf_reader);
    let decompressed_stream = ReaderStream::new(gzip).map_ok(Bytes::from);
    Body::from_stream(decompressed_stream)
}

async fn fetch_url(
    url: hyper::Uri,
    req: http::Request<axum::body::Body>,
) -> Result<axum::response::Response<axum::body::Body>, Box<dyn std::error::Error>> {
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
    req_builder = req_builder.method(req.method());

    for (k, v) in req.headers().iter() {
        req_builder = req_builder.header(k, v);
    }

    let req0 = if req.headers().contains_key(CUSTOM_ENCODING_HEADER) {
        req_builder.headers_mut().unwrap().remove(CONTENT_ENCODING);
        req_builder
            .headers_mut()
            .unwrap()
            .remove(CUSTOM_ENCODING_HEADER);
        req_builder.headers_mut().unwrap().remove(TRANSFER_ENCODING);
        if let Some(length) = req_builder.headers_mut().unwrap().remove(CUSTOM_LENGTH) {
            req_builder = req_builder.header(CONTENT_LENGTH, length);
        }

        req_builder.body(maybe_decompress_body(
            &req.method().clone(),
            req.into_body(),
        ))?
    } else {
        req_builder = req_builder
            .header(CUSTOM_ENCODING_HEADER, CUSTOM_ENCODING_VALUE)
            .header(CONTENT_ENCODING, CUSTOM_ENCODING_VALUE);
        if let Some(length) = req_builder.headers_mut().unwrap().remove(CONTENT_LENGTH) {
            req_builder = req_builder.header(CUSTOM_LENGTH, length);
        }
        // println!("headers:{:?}", req.headers());
        let req2: Request<Body> =
            req_builder.body(maybe_compress_body(&req.method().clone(), req.into_body()))?;
        // println!("req:{:?}", req2);
        req2
    };

    let mut res = timeout(Duration::from_secs(600), sender.send_request(req0)).await??;

    let (parts, body_stream) = res.into_parts();
    let mut  data_stream = body_stream.into_data_stream();

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

pub fn init_log4rs_rolling(log_path: &str, max_size_mb: u64, max_files: u32) {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S%.3f)} [{l}] {t} - {m}{n}",
        )))
        .build();

    let trigger = SizeTrigger::new(max_size_mb * 1024 * 1024);

    let roller = FixedWindowRoller::builder()
        .build(&format!("{}.{{}}", log_path), max_files)
        .unwrap();

    let policy = CompoundPolicy::new(Box::new(trigger), Box::new(roller));

    let rolling = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S%.3f)} [{l}] {t} - {m}{n}",
        )))
        .build(log_path, Box::new(policy))
        .unwrap();

    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("rolling", Box::new(rolling)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("rolling")
                .build(LevelFilter::Info),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();
}
