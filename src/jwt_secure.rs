use rocket::Responder;

use rocket::serde::{Deserialize, Serialize};

use chrono::Utc;
use jsonwebtoken::{encode, decode, DecodingKey, Algorithm, Header, Validation}; // 👈 New!
use jsonwebtoken::errors::{Error, ErrorKind};
use std::env;
use std::fs::File;
use dotenvy::dotenv;
use std::io::Read;
//use shared::response_models::{Response, ResponseBody, NetworkResponse}; // 👈 New!
use rocket::request::{Outcome, Request, FromRequest}; // 👈 New!
use rocket::http::Status;

#[derive(Debug)]
pub struct JWT {
    pub claims: Claims
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JWT {
    type Error = NetworkResponse;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, NetworkResponse> {
        fn is_valid(key: &str) -> Result<Claims, Error> {
            Ok(decode_jwt(String::from(key))?)
        }

        match req.headers().get_one("authorization") {
            None => {
                let response = Response { body: ResponseBody::Message(String::from("Error validating JWT token - No token provided"))};

                println!("Error validating JWT token - No token provided");
                Outcome::Failure((Status::Unauthorized, NetworkResponse::Unauthorized(serde_json::to_string(&response).unwrap())))
            },
            Some(key) => match is_valid(key) {
                Ok(claims) => Outcome::Success(JWT {claims}),
                Err(err) => match &err.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        let response = Response { body: ResponseBody::Message(format!("Error validating JWT token - Expired Token"))};
                        println!("Error validating JWT token - Expired Token");
                        Outcome::Failure((Status::Unauthorized, NetworkResponse::Unauthorized(serde_json::to_string(&response).unwrap())))
                    },
                    jsonwebtoken::errors::ErrorKind::InvalidToken => {
                        let response = Response { body: ResponseBody::Message(format!("Error validating JWT token - Invalid Token"))};
                        // Print the error to the console
                        println!("Error validating JWT token - Invalid Token");
                        Outcome::Failure((Status::Unauthorized, NetworkResponse::Unauthorized(serde_json::to_string(&response).unwrap())))
                    },
                    _ => {
                        let response = Response { body: ResponseBody::Message(format!("Error validating JWT token - {}", err))};
                        println!("{}",format!("Error validating JWT token - {}", err));
                        Outcome::Failure((Status::Unauthorized, NetworkResponse::Unauthorized(serde_json::to_string(&response).unwrap())))
                    }
                }
            },
        }
    }
}

    fn decode_jwt(token: String) -> Result<Claims, ErrorKind> {
        //let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set.");
        let token = token.trim_start_matches("Bearer").trim();

        let mut pem = Vec::new();
        File::open("rsapk.pem")
            .expect("Unable to open PEM file")
            .read_to_end(&mut pem)
            .expect("Unable to read PEM file");

        // 👇 New!
        match decode::<Claims>(
            &token,
            &DecodingKey::from_rsa_pem(&pem).unwrap(),
            &Validation::new(Algorithm::RS256),
        ) {
            Ok(token) => Ok(token.claims),
            Err(err) => Err(err.kind().to_owned())
        }
    }

#[derive(Responder, Debug)]
pub enum NetworkResponse {
    #[response(status = 201)]
    Created(String),
    #[response(status = 400)]
    BadRequest(String),
    #[response(status = 401)]
    Unauthorized(String),
    #[response(status = 404)]
    NotFound(String),
    #[response(status = 409)]
    Conflict(String),
}

#[derive(Serialize)]
pub enum ResponseBody {
    Message(String),
    AuthToken(String),
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Response {
    pub body: ResponseBody,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub iat: i32,
    scope: String
}
