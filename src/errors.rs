use rocket::request::Request;
use rocket::response::{self, Responder};
use std::fmt;

#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
    Uncancelable,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NotFound => write!(f, "Resource could not be found"),
            Error::Uncancelable => write!(f, "Target order is no longer cancelable"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl<'r> Responder<'r> for Error {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        match self {
            Error::NotFound => Err(rocket::http::Status::NotFound),
            Error::Uncancelable => Err(rocket::http::Status::UnprocessableEntity),
        }
    }
}
