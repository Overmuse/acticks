use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use rocket::Rocket;
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Serialize, Debug, PartialEq)]
pub struct Credentials {
    pub key_id: Uuid,
    pub secret_key: Uuid,
}

impl Credentials {
    pub fn new() -> Self {
        Self {
            key_id: Uuid::new_v4(),
            secret_key: Uuid::new_v4(),
        }
    }
}

#[derive(Debug)]
pub enum CredentialsError {
    Missing,
    Invalid,
}

impl<'a, 'r> FromRequest<'a, 'r> for Credentials {
    type Error = CredentialsError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let id: Vec<_> = request.headers().get("APCA-API-KEY-ID").collect();
        let secret_key: Vec<_> = request.headers().get("APCA-API-SECRET-KEY").collect();

        match (id.len(), secret_key.len()) {
            (1, 1) => Outcome::Success(Credentials {
                key_id: Uuid::parse_str(id[0]).unwrap(),
                secret_key: Uuid::parse_str(secret_key[0]).unwrap(),
            }),
            _ => Outcome::Failure((Status::BadRequest, CredentialsError::Invalid)),
        }
    }
}

impl Fairing for Credentials {
    fn info(&self) -> Info {
        Info {
            name: "Credentials",
            kind: Kind::Launch,
        }
    }

    fn on_launch(&self, _rocket: &Rocket) {
        println!("{:#?}", self);
    }
}
