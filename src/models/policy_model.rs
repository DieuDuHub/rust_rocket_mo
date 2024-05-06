use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// User Rust structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Policy {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub content: serde_json::Value,
}
