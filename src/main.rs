use async_trait::async_trait;
use bytes::Bytes;
use clap::Parser;
use flate2::{GzBuilder, write::GzEncoder};
use http::header::{ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_LENGTH, TRANSFER_ENCODING};
use http_proxy::compress::{Compressor, Decompressor, Encode};
use http_proxy::config::{self, Config};
use pingora::{
    Result,
    http::{RequestHeader, ResponseHeader},
    prelude::{HttpPeer, Opt},
    proxy::{ProxyHttp, Session},
    server::Server,
};
use std::{env, sync::Arc};

fn main() {
    env_logger::init();
    let config = Config::parse();
    let opt = Opt::default();
    let mut my_server = Server::new(Some(opt)).unwrap();
    my_server.bootstrap();
    let mut my_proxy = pingora::proxy::http_proxy_service(&Arc::new(Default::default()), Proxy0 { config: config.clone() });
    my_proxy.add_tcp(&format!(
        "0.0.0.0:{}", config.port
    ));
    my_server.add_service(my_proxy);
    my_server.run_forever();
}

pub struct ProxyCtx {
    op: Op,
    compressor: Option<Compressor>,
    decompressor: Option<Decompressor>,
}

pub enum Op {
    None,
    Compress,
    Decompress,
}

pub struct Proxy0 {
    config: Config,
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
            ctx.compressor = Some(Compressor::new(3));
            if let Some(cl) = upstream_request.remove_header(&CONTENT_LENGTH) {
                upstream_request.insert_header("crd-content-length", cl);
            }
            upstream_request.insert_header(CONTENT_ENCODING, "gzip");
            upstream_request.insert_header(TRANSFER_ENCODING, "Chunked");
        } else {
            ctx.op = Op::Decompress;
            ctx.decompressor = Some(Decompressor::new());
            upstream_request.insert_header(ACCEPT_ENCODING, "gzip");
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

        // let data = if let Some(b) = body.as_ref() {
        //     b.as_ref()
        // } else {
        //     &[]
        // };
        // println!("data len:{:?}, {:?}", data, String::from_utf8_lossy(data));
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

    // fn upstream_response_filter(
    //     &self,
    //     session: &mut Session,
    //     _upstream_response: &mut ResponseHeader,
    //     _ctx: &mut Self::CTX,
    // ) -> Result<()> {

    //     Ok(())
    // }

    fn upstream_response_filter(
        &self,
        session: &mut Session,
        upstream_response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        // println!("upstream_response_filter");
        // session
        //     .upstream_compression
        //     .response_header_filter(upstream_response, false);
        Ok(())
    }

    fn upstream_response_body_filter(
        &self,
        session: &mut Session,
        body: &mut Option<Bytes>,
        end_of_stream: bool,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        // println!("upstream_response_body_filter");
        // println!("Compress:{:?}", session.upstream_compression.is_enabled());
        // println!("body:{:?}", body);
        // *body = session
        //     .upstream_compression
        //     .response_body_filter(body.as_ref(), end_of_stream);
        // println!("body1:{:?}", body);
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
