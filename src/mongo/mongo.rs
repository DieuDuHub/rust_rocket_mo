use bson::Document;
#[allow(unused_imports)] // need for mongo connection
use futures::stream::StreamExt;
use mongodb::{
    bson::doc,
    options::{ClientOptions, ServerApi, ServerApiVersion},
    Client, Collection,
};

use crate::models::user_model::User;

/// Collection structure to CRUD User object
/// MongoDb shared connection
pub struct MongoRepo {
    pub user_col: Collection<User>,
    pub policy_col: Collection<Document>,
    pub history_col: Collection<Document>,
    pub deleted_col: Collection<Document>,
    pub repo: mongodb::Client,
}

/// Initialize DB object containings the tested, alive mongodb::Client instance
///
pub async fn init_connection(uri: String) -> Option<MongoRepo> {
    let mut client_options = match ClientOptions::parse(uri).await {
        Ok(o) => o,
        Err(e) => {
            println!("Error init connection client options {}", e.to_string());
            return None;
        }
    };
    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);
    // Create a new client and connect to the server
    let client = match Client::with_options(client_options) {
        Ok(o) => o,
        Err(e) => {
            println!("Error init connection server api {}", e.to_string());
            return None;
        }
    };
    // Send a ping to confirm a successful connection
    match client
        .database("middleoffice")
        .run_command(doc! { "ping": 1 }, None)
        .await
    {
        Ok(_o) => {
            let user_col = client.database("middleoffice").collection("Users");
            let policy_col = client.database("middleoffice").collection("policies");
            let history_col = client.database("middleoffice").collection("history");
            let deleted_col = client.database("middleoffice").collection("deleted");
            let repo = client; //.database("middleoffice");
            return Some(MongoRepo {
                user_col,
                policy_col,
                history_col,
                deleted_col,
                repo,
            });
        } // don't care about the document but connection is validated
        Err(e) => {
            println!("Error init collection{}", e.to_string());
            return None;
        }
    };
}

/// ping_db : Create a client connection t mongo db Uri and execute a ping request
/// Use also for reconnect
/// return Ok() if succesfull, Error if not
pub async fn ping_db(db: &MongoRepo) -> mongodb::error::Result<()> {
    // Send a ping to confirm a successful connection
    db.repo
        .database("middleoffice")
        .run_command(doc! { "ping": 1 }, None)
        .await?;

    Ok(())
}
