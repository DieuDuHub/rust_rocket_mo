#[macro_use]
extern crate rocket;
use std::process;

use rocket::data::ToByteUnit;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::Data;
use rocket::State;

mod config;
mod models;
mod mongo;
#[macro_use]
mod error;

#[cfg(test)]
mod test;
mod jwt_secure;

use crate::models::policy_model::Policy;
use crate::models::user_model::User;
use crate::mongo::mongo::MongoRepo;

use mongodb::results::InsertOneResult;
use serde_json::json;

use mongo::mongo_any;
use mongo::mongo_users;
use crate::jwt_secure::{JWT, NetworkResponse};


/// Getter for the /ping URI. allow to execute and ping DB connection each times method is get
#[get("/api/ping")]
async fn ping(db: &State<MongoRepo>) -> String {
    match mongo::mongo::ping_db(&db).await {
        Ok(()) => "Pinged your deployment. You successfully connected to MongoDB!".to_string(),
        Err(e) => format!("Error ping db return error: {}", e.to_string()),
    }
}

/// Getter for the /ping URI. allow to execute and ping DB connection each times method is get
#[post("/api/user", data = "<user>")]
async fn post_user(
    db: &State<MongoRepo>,
    user: Json<User>,
) -> Result<Json<InsertOneResult>, Status> {
    let data = User {
        id: None,
        name: user.name.to_owned(),
        location: user.location.to_owned(),
        title: user.title.to_owned(),
    };

    let result = mongo_users::create_user(db, data).await;
    match result {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Retrieve a User from an MongoDB Atlas OID.
#[get("/api/user/<path>")]
async fn get_user(db: &State<MongoRepo>, path: String) -> Result<Json<User>, Status> {
    let id = path;
    if id.is_empty() {
        return Err(Status::BadRequest);
    }
    let user_detail = mongo_users::get_user(db, &id).await;
    match user_detail {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Retrieve all Users API
#[get("/api/users")]
async fn get_users(db: &State<MongoRepo>) -> Result<Json<Vec<User>>, Status> {
    let users = mongo_users::get_all_users(db).await;
    match users {
        Ok(users) => Ok(Json(users)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[delete("/api/user/<path>")]
async fn delete_user(db: &State<MongoRepo>, path: String) -> Result<Json<&str>, Status> {
    let id = path;
    if id.is_empty() {
        return Err(Status::BadRequest);
    }
    let result = mongo_users::delete_user(db, &id).await;
    match result {
        Ok(res) => {
            if res.deleted_count == 1 {
                return Ok(Json("User successfully deleted!"));
            } else {
                return Err(Status::NotFound);
            }
        }
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Post Any
#[post("/api/any", data = "<any>")]
async fn post_any(db: &State<MongoRepo>, any: Data<'_>) -> Result<Json<serde_json::Value>, Status> {
    let body = any.open(2.mebibytes()).into_string().await;

    let request: serde_json::Value = match serde_json::from_str(&body.unwrap()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error {:?}", e);
            return Err(Status::BadRequest);
        }
    };

    let result = mongo_any::create_any(db, request).await;
    match result {
        Ok(policy) => Ok(Json(policy.content)),
        Err(e) => Ok(Json(json!({"exception" : e.to_string()}))),
    }
}

/// Get any from Oid
#[get("/api/any/<path>")]
async fn get_any(db: &State<MongoRepo>, path: String) -> Result<Json<serde_json::Value>, Status> {
    let id = path;
    if id.is_empty() {
        return Err(Status::BadRequest);
    }

    let result = mongo_any::get_any(db, &id).await;
    match result {
        Ok(policy) => Ok(Json(policy.content)),
        Err(e) => Ok(Json(json!({"exception" : e.to_string()}))),
    }
}

/// Update any from oid
#[put("/api/any/<path>")]
async fn update_any_empty(
    db: &State<MongoRepo>,
    path: String,
) -> Result<Json<serde_json::Value>, Status> {
    let id = path;
    if id.is_empty() {
        return Err(Status::BadRequest);
    }

    let result = mongo_any::update_any(db, json!({}), id).await;
    match result {
        Ok(policy) => Ok(Json(policy.content)),
        Err(e) => Ok(Json(json!({"exception" : e.to_string()}))),
    }
}

/// Update any from oid
#[put("/api/any/<path>", data = "<any>")]
async fn update_any(
    db: &State<MongoRepo>,
    path: String,
    any: Data<'_>,
) -> Result<Json<serde_json::Value>, Status> {
    let id = path;
    if id.is_empty() {
        return Err(Status::BadRequest);
    }

    let body = any.open(2.mebibytes()).into_string().await;

    let request: serde_json::Value = match serde_json::from_str(&body.unwrap()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error {:?}", e);
            return Err(Status::BadRequest);
        }
    };

    let result = mongo_any::update_any(db, request, id).await;
    match result {
        Ok(policy) => Ok(Json(policy.content)),
        Err(e) => Ok(Json(json!({"exception" : e.to_string()}))),
    }
}

/// Retrieve all Any API
#[get("/api/anys?<date>&<policyholder>&<page>&<limit>")]
async fn get_all_any(
    db: &State<MongoRepo>,
    date: Option<String>,
    policyholder: Option<String>,
    page: Option<i64>,
    limit: Option<i64>,
    key: Result<JWT, NetworkResponse>
) -> Result<Json<Vec<Policy>>, Status> {

    let key = match key {
        Ok(key) => key,
        Err(err) => {
            //println!("Error init connection server api {}", err.);
            //return Err(Status::BadRequest);
            match (err) {
                NetworkResponse::Unauthorized(e) => {
                    return Err(Status::Unauthorized);
                },
                _ => {
                    return Err(Status::BadRequest);
                }
            }
        },
    };

    let pagev = match page {
        Some(o) => o,
        None => 1,
    };

    let limitv = match limit {
        Some(o) => o,
        None => 10,
    };

    let records = mongo_any::get_all_any(db, date, policyholder, (pagev, limitv)).await;

    match records {
        Ok(record) => Ok(Json(record)),
        Err(e) => {
            eprintln!("Get anys by filter Error : {}", e);
            return Err(Status::BadRequest);
        }
    }
}

/// Count all Any API
#[get("/api/countanys?<date>&<policyholder>")]
async fn count_all_any(
    db: &State<MongoRepo>,
    date: Option<String>,
    policyholder: Option<String>,
) -> Result<Json<serde_json::Value>, Status> {
    let records = mongo_any::count_all_any(db, date, policyholder).await;

    match records {
        Ok(record) => Ok(Json(json!({"result":record}))),
        Err(e) => {
            eprintln!("Get anys by filter Error : {}", e);
            return Err(Status::BadRequest);
        }
    }
}

/// Delete policy to an input Oid
#[delete("/api/any/<path>")]
async fn delete_any(
    db: &State<MongoRepo>,
    path: String,
) -> Result<Json<serde_json::Value>, Status> {
    let id = path;
    if id.is_empty() {
        return Err(Status::BadRequest);
    }
    let result = mongo_any::delete_any(db, &id).await;
    match result {
        Ok(res) => {
            if res.deleted_count == 1 {
                return Ok(Json(json!({"result" : "Policy successfully deleted!"})));
            } else {
                return Ok(Json(json!({"exception" : "No result.."})));
            }
        }
        Err(e) => Ok(Json(json!({"exception" : e.to_string()}))),
    }
}

/// Main start routines.
#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let settings = config::init_configuration(String::from("")).await.unwrap();
    let uri = match settings.get::<String>("db_uri_atlas") {
        Ok(o) => o,
        Err(e) => format!("Error ping db get Key return error: {}", e.to_string()),
    };
    
    let repo: MongoRepo = match mongo::mongo::init_connection(uri).await {
        Some(o) => o,
        None => {
            eprintln!("Error affecting client DB in init.");
            process::exit(1);
        }
    };

    let _rocket = rocket::build()
        .manage(repo)
        .mount(
            "/",
            routes![
                ping,
                post_user,
                get_user,
                get_users,
                delete_user,
                post_any,
                get_any,
                get_all_any,
                count_all_any,
                delete_any,
                update_any
            ],
        )
        .launch()
        .await?;

    Ok(())
}
