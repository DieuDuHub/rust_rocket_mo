use crate::models::user_model::User;
use futures::stream::StreamExt;
use mongodb::{
    bson::doc, bson::oid::ObjectId, error::Error, results::DeleteResult, results::InsertOneResult,
};

use crate::mongo::mongo::MongoRepo;

/// Create User API, return Mongo Oid on success, Err(e) if exception.
/// Json User data from : { "name" : "toto", "location":"paris","title":"architect" }
pub async fn create_user(db: &MongoRepo, new_user: User) -> Result<InsertOneResult, Error> {
    let new_doc = User {
        id: None,
        name: new_user.name,
        location: new_user.location,
        title: new_user.title,
    };

    let user = db
        .user_col
        .insert_one(new_doc, None)
        .await
        .ok()
        .expect("Error creating user on atlas.");

    Ok(user)
}

/// Return signle User from a get Oid
pub async fn get_user(db: &MongoRepo, id: &String) -> Result<User, Error> {
    let obj_id = ObjectId::parse_str(id).unwrap();
    let filter = doc!("_id": obj_id);
    let user_detail = db
        .user_col
        .find_one(filter, None)
        .await
        .ok()
        .expect("Error getting user's detail");
    Ok(user_detail.unwrap())
}

/// Return list of users without filtering/pagination
pub async fn get_all_users(db: &MongoRepo) -> Result<Vec<User>, Error> {
    let cursors = db
        .user_col
        .find(None, None)
        .await
        .ok()
        .expect("Error getting list of users");
    let users = cursors.map(|doc| doc.unwrap()).collect().await;
    Ok(users)
}

/// Delete users based on an Oid
pub async fn delete_user(db: &MongoRepo, id: &String) -> Result<DeleteResult, Error> {
    let obj_id = ObjectId::parse_str(id).unwrap();
    let filter = doc!("_id": obj_id);
    let user_detail = db
        .user_col
        .delete_one(filter, None)
        .await
        .ok()
        .expect("Error deleting user");
    Ok(user_detail)
}
