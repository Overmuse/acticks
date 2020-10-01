use actix_web::{dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("resource not found")]
    NotFound,

    #[error("target order is no longer cancelable")]
    Uncancelable,

    #[error("tried to get uninitialize price")]
    UninitializedPrice,

    #[error("Missing environment variable: {0}")]
    MissingEnv(#[from] std::env::VarError),

    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Actix(#[from] actix::MailboxError),

    #[error("internal error")]
    Other,
}

pub type Result<T> = std::result::Result<T, Error>;

impl error::ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::Uncancelable => StatusCode::UNPROCESSABLE_ENTITY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
