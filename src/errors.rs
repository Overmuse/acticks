use actix_web::{dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse};
use derive_more::{Display, Error};

#[derive(Debug, Clone, Display, Error)]
pub enum Error {
    #[display(fmt = "resource not found")]
    NotFound,
    #[display(fmt = "target order is no longer cancelable")]
    Uncancelable,
    #[display(fmt = "internal error")]
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
            Error::Other => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
