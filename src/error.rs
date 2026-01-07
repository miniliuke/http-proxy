use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("io error")]
    IO(#[from] io::Error),
    #[error("sqlx error")]
    SQLX(#[from] sqlx::Error),
    #[error("hyper error")]
    Hyper(#[from] hyper::Error),
    #[error("TIMEOUT")]
    Timeout,
    #[error("common error")]
    Common(String),
    #[error("common error")]
    Common0(&'static str),
    #[error("unknown data store error")]
    Unknown,
}
