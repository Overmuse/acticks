use rocket::request::Request;
use rocket::response::{self, Responder, Response};

#[derive(Debug, Clone)]
pub enum Error {
    NotFound { msg: String },
}
pub type Result<T> = std::result::Result<T, Error>;

impl<'r> Responder<'r> for Error {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        Err(rocket::http::Status::NotFound)
    }
}
