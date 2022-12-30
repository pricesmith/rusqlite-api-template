use std::fmt;
use std::task::{Context, Poll};
// use actix_web::http::StatusCode;
use actix_web::web;
use actix_web::body::{MessageBody, BodySize};

#[derive(Debug)]
pub enum ServerError {
    Io(std::io::Error),
    Actix(actix_web::Error),
    Rusqlite(rusqlite::Error),
    Other(String),
}

impl std::error::Error for ServerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ServerError::Io(e) => Some(e),
            ServerError::Actix(e) => Some(e),
            ServerError::Rusqlite(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ServerError::Io(e) => write!(f, "I/O error: {}", e),
            ServerError::Actix(e) => write!(f, "Actix error: {}", e),
            ServerError::Rusqlite(e) => write!(f, "Rusqlite error: {}", e),
            ServerError::Other(s) => write!(f, "Other error: {}", s),
        }
    }
}

impl MessageBody for ServerError {
    // ?: Examples show Error definitions, but this doesn't appear to work. 
    // type Error = actix_web::Error;

    fn size(&self) -> BodySize { 
        // Since the body size of the error response is variable, I think
        // we can return `BodySize::None` to indicate that the size isn't known. 
        // This would Actix-Web to skip reading the body in some cases. 
        BodySize::None
    }

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<web::Bytes, actix_web::Error>>> {
        // Since the `ServerError` type is an enumeration of different error types,
        // you can create a response body by simply converting the `ServerError`
        // into a string and returning it as a single chunk of bytes.
        let body = format!("{}", self);
        let bytes = web::Bytes::from(body);
        Poll::Ready(Some(Ok(bytes)))
    }
}