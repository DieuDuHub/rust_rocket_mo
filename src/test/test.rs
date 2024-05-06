#![allow(dead_code)]
//#![allow(unused_variables)]

#[cfg(test)]
use crate::{delete_any, get_all_any, get_any, update_any};
use rocket::serde::Deserialize;

use crate::rocket;
use rocket::local::asynchronous::Client;
use rocket::serde::json::json;

use crate::config;
use crate::mongo;
use std::process;

#[launch]
async fn rocket() -> _ {
    let settings = config::init_configuration(String::from("")).await.unwrap();
    let uri = match settings.get::<String>("db_uri_atlas") {
        Ok(o) => o,
        Err(e) => format!("Error ping db get Key return error: {}", e.to_string()),
    };

    let repo: mongo::mongo::MongoRepo = match mongo::mongo::init_connection(uri).await {
        Some(o) => o,
        None => {
            eprintln!("Error affecting client DB in init.");
            process::exit(1);
        }
    };

    //rocket::build().manage(repo).mount("/", routes![get_any])
    rocket::build()
        .manage(repo)
        .mount("/", routes![get_any, get_all_any, update_any, delete_any])
}

/// All exception shoulbd be prefix by a lower case expression as detailled object.
/// ie : {"exception":"this is the deailled exception"}
#[derive(Deserialize)]
struct Except {
    pub exception: String,
}

#[async_test]
async fn get_api_oid_error() {
    let client = Client::tracked(rocket().await);
    let binding = client.await.unwrap();
    let response = binding.get("/api/any/1234").dispatch();

    assert_eq!(
        response
            .await
            .into_json::<Except>()
            .await
            .unwrap()
            .exception,
        "ObjectId exception : ObjectId wrongly structure."
    );
}

#[async_test]
async fn get_api_oid_not_found() {
    let client = Client::tracked(rocket().await);
    let binding = client.await.unwrap();
    let response = binding.get("/api/any/655c7c5b037c912bb7ce3973").dispatch();

    assert_eq!(
        response
            .await
            .into_json::<Except>()
            .await
            .unwrap()
            .exception,
        "Data not found : No result."
    );
}

#[async_test]
async fn get_api_delete_not_found() {
    let client = Client::tracked(rocket().await);
    let binding = client.await.unwrap();
    let response = binding
        .delete("/api/any/654a03b71dd74c6443810c2f")
        .dispatch();

    assert_eq!(
        response
            .await
            .into_json::<Except>()
            .await
            .unwrap()
            .exception,
        "Data not found : No result."
    );
}

#[async_test]
async fn get_api_put_not_found() {
    let client = Client::tracked(rocket().await);
    let binding = client.await.unwrap();
    let body = json!({
        "source": "Dummy",
    });
    let response = binding
        .put("/api/any/655c7c41037c412bb7ce6971")
        .body(body.to_string())
        .dispatch();

    assert_eq!(
        response
            .await
            .into_json::<Except>()
            .await
            .unwrap()
            .exception,
        "Connection exception : Get policy failed: Data not found : No result.."
    );
}

#[async_test]
async fn test_ping_db_validate_configuration() {
    let settings = config::init_configuration(String::from("")).await.unwrap();
    let uri = match settings.get::<String>("db_uri_atlas") {
        Ok(o) => o,
        Err(e) => format!("Error ping db get Key return error: {}", e.to_string()),
    };

    let db = match mongo::mongo::init_connection(uri).await {
        Some(o) => o,
        None => {
            eprintln!("Error affecting client DB in init.");
            process::exit(1);
        }
    };

    assert_eq!(mongo::mongo::ping_db(&db).await.unwrap(), ());
}

#[async_test]
async fn test_get_any_path() {
    let settings = config::init_configuration(String::from("")).await.unwrap();
    let uri = match settings.get::<String>("db_uri_atlas") {
        Ok(o) => o,
        Err(e) => format!("Error ping db get Key return error: {}", e.to_string()),
    };

    let db = match mongo::mongo::init_connection(uri).await {
        Some(o) => o,
        None => {
            eprintln!("Error affecting client DB in init.");
            process::exit(1);
        }
    };

    assert_eq!(mongo::mongo::ping_db(&db).await.unwrap(), ());
}
