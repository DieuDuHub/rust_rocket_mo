use rocket::Responder;

use rocket::serde::{Deserialize, Serialize};

use jsonwebtoken::{encode, decode, DecodingKey, Algorithm, Header, Validation}; // 👈 New!
use jsonwebtoken::errors::{Error, ErrorKind};

use std::fs::File;

use std::io::Read;
//use shared::response_models::{Response, ResponseBody, NetworkResponse}; // 👈 New!
use rocket::request::{Outcome, Request, FromRequest}; // 👈 New!
use rocket::http::Status;
use reqwest::get;
use std::env;

#[derive(Debug)]
pub struct JWT {
    pub claims: Claims,
}

#[derive(Deserialize,Debug,Serialize)]
struct Jwk {
    n: String,
    e: String,
}

#[derive(Deserialize,Debug,Serialize)]
struct JwkSet {
    keys: Vec<Jwk>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JWT {
    type Error = NetworkResponse;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, NetworkResponse> {
        async fn is_valid(key: &str) -> Result<Claims, Error> {
            Ok(decode_jwt(String::from(key)).await?)
        }

        match req.headers().get_one("authorization") {
            None => {
                let response = Response { body: ResponseBody::Message(String::from("Error validating JWT token - No token provided"))};

                println!("Error validating JWT token - No token provided");
                Outcome::Failure((Status::Unauthorized, NetworkResponse::Unauthorized(serde_json::to_string(&response).unwrap())))
            },
            Some(key) => match is_valid(key).await {
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

   async fn decode_jwt(token: String) -> Result<Claims, ErrorKind> {
       
        let token = token.trim_start_matches("Bearer").trim();

        let secret = env::var("JWT_HOST").expect("JWT_HOST must be set.");
        // Effectuer une requête HTTP pour obtenir le Certs
        //println!("JWT URL : {}",secret);
        let jwk_set  = match get(format!("{}{}","http://",secret)).await {
            Ok(response) => response,
            Err(err) => {
                println!("Error fetching PEM file: {:?}", err);
                return Err(ErrorKind::InvalidToken);
            }   
        };

        let jwk_json : JwkSet  = match jwk_set.json().await {
            Ok(response) => response,
            Err(err) => {
                println!("Error fetching to JSON file: {:?}", err);
                return Err(ErrorKind::InvalidToken);
            }   
        };

       // println!("Bearer {}",&token);

       // let pem = response.as_bytes();
       let jwk = &jwk_json.keys[0];

        // 👇 New!
        match decode::<Claims>(
            &token,
            //&DecodingKey::from_jwk(&pem).unwrap(),
            &DecodingKey::from_rsa_components(&jwk.n, &jwk.e).unwrap(),
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
    scope: String,
}
