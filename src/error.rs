use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("io error")]
    IO(#[from] io::Error),
    #[error("hyper error")]
    Hyper(#[from] hyper::Error),
    #[error("TIMEOUT")]
    Timeout,
    #[error("unknown data store error")]
    Unknown,
}