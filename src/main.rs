use async_trait::async_trait;
use bytes::Bytes;
use clap::Parser;
use flate2::{GzBuilder, write::GzEncoder};
use http::header::{ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_LENGTH, TRANSFER_ENCODING};
use http_proxy::compress::{Compressor, Decompressor, Encode, ZstdCompressor, ZstdDecompressor};
use http_proxy::config::{self, Config};
use pingora::server::configuration::ServerConf;
use pingora::{
    Result,
    http::{RequestHeader, ResponseHeader},
    prelude::{HttpPeer, Opt},
    proxy::{ProxyHttp, Session},
    server::Server,
};
use std::fs::File;
use std::io::Write;
use std::{env, sync::Arc};

fn main() {
    env_logger::init();
    let server_conf = ServerConf {
        threads: 128,
        listener_tasks_per_fd: 2,
        ..Default::default()
    };

    let config = Config::parse();
    let mut opt = Opt::default();
    if let Ok(mut file) = File::create("config.yaml") {
        let _ = file.write_all(server_conf.to_yaml().as_bytes());
        let _ = file.flush();
    }
    opt.conf = Some("config.yaml".to_string());
    let mut my_server = Server::new(Some(opt)).unwrap();
    my_server.bootstrap();
    let mut my_proxy = pingora::proxy::http_proxy_service(
        &Arc::new(server_conf),
        Proxy0 {
            config: config.clone(),
            zstd: true,
        },
    );
    my_proxy.add_tcp(&format!("0.0.0.0:{}", config.port));
    my_server.add_service(my_proxy);
    my_server.run_forever();
}

pub enum Compreessor0 {
    Gzip(Compressor),
    Zstd(ZstdCompressor),
}

impl Compreessor0 {
    fn encode(&mut self, input: &[u8], end: bool) -> Result<Bytes> {
        match self {
            Compreessor0::Gzip(compressor) => compressor.encode(input, end),
            Compreessor0::Zstd(zstd_compressor) => zstd_compressor.encode(input, end),
        }
    }
}

pub enum Decompreessor0 {
    Gzip(Decompressor),
    Zstd(ZstdDecompressor),
}

impl Decompreessor0 {
    fn encode(&mut self, input: &[u8], end: bool) -> Result<Bytes> {
        match self {
            Decompreessor0::Gzip(compressor) => compressor.encode(input, end),
            Decompreessor0::Zstd(zstd_compressor) => zstd_compressor.encode(input, end),
        }
    }
}

pub struct ProxyCtx {
    op: Op,
    compressor: Option<Compreessor0>,
    decompressor: Option<Decompreessor0>,
}

pub enum Op {
    None,
    Compress,
    Decompress,
}

pub struct Proxy0 {
    config: Config,
    zstd: bool,
}

#[async_trait]
impl ProxyHttp for Proxy0 {
    type CTX = ProxyCtx;

    fn new_ctx(&self) -> Self::CTX {
        ProxyCtx {
            op: Op::None,
            compressor: None,
            decompressor: None,
        }
    }

    async fn upstream_peer(
        &self,
        session: &mut pingora::prelude::Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        Ok(Box::new(HttpPeer::new(
            &self.config.target,
            false,
            "one".to_string(),
        )))
    }

    async fn request_filter(&self, session: &mut Session, _ctx: &mut Self::CTX) -> Result<bool>
    where
        Self::CTX: Send + Sync,
    {
        // println!("Header:{:?}", session.as_downstream().req_header());
        Ok(false)
    }

    async fn upstream_request_filter(
        &self,
        session: &mut Session,
        upstream_request: &mut RequestHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()>
    where
        Self::CTX: Send + Sync,
    {
        if let None = upstream_request.headers.get(CONTENT_ENCODING) {
            ctx.op = Op::Compress;

            if let Some(cl) = upstream_request.remove_header(&CONTENT_LENGTH) {
                upstream_request.insert_header("crd-content-length", cl);
            }
            if self.zstd {
                upstream_request.insert_header(CONTENT_ENCODING, "zstd");
                ctx.compressor = Some(Compreessor0::Zstd(ZstdCompressor::new(6)));
            } else {
                upstream_request.insert_header(CONTENT_ENCODING, "gzip");
                ctx.compressor = Some(Compreessor0::Gzip(Compressor::new(6)));
            }

            upstream_request.insert_header(TRANSFER_ENCODING, "Chunked");
        } else {
            ctx.op = Op::Decompress;
            if self.zstd {
                ctx.decompressor = Some(Decompreessor0::Zstd(ZstdDecompressor::new()));
                upstream_request.insert_header(ACCEPT_ENCODING, "zstd");
            } else {
                ctx.decompressor = Some(Decompreessor0::Gzip(Decompressor::new()));
                upstream_request.insert_header(ACCEPT_ENCODING, "gzip");
            }

            if let Some(cl) = upstream_request.headers.get("crd-content-length") {
                upstream_request.insert_header(CONTENT_LENGTH, cl.clone());
                upstream_request.remove_header(&TRANSFER_ENCODING);
            }
        }

        session.upstream_compression.adjust_decompression(true);
        Ok(())
    }

    async fn request_body_filter(
        &self,
        session: &mut Session,
        body: &mut Option<Bytes>,
        end: bool,
        ctx: &mut Self::CTX,
    ) -> Result<()>
    where
        Self::CTX: Send + Sync,
    {
        if let Some(compresser) = ctx.compressor.as_mut() {
            let data = if let Some(b) = body.as_ref() {
                b.as_ref()
            } else {
                &[]
            };
            *body = Some(compresser.encode(data, end)?);
        }

        if let Some(decompressor) = ctx.decompressor.as_mut() {
            let data = if let Some(b) = body.as_ref() {
                b.as_ref()
            } else {
                &[]
            };
            *body = Some(decompressor.encode(data, end)?);
        }
        Ok(())
    }

    fn upstream_response_filter(
        &self,
        session: &mut Session,
        upstream_response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        Ok(())
    }

    fn upstream_response_body_filter(
        &self,
        session: &mut Session,
        body: &mut Option<Bytes>,
        end_of_stream: bool,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        Ok(())
    }

    fn response_body_filter(
        &self,
        session: &mut Session,
        body: &mut Option<Bytes>,
        end_of_stream: bool,
        ctx: &mut Self::CTX,
    ) -> Result<Option<std::time::Duration>>
    where
        Self::CTX: Send + Sync,
    {
        Ok(None)
    }
}
