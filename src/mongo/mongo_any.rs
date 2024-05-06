use mongodb::{bson::doc, bson::oid::ObjectId, results::DeleteResult};

use futures::stream::StreamExt;

use crate::models::policy_model::Policy;
use mongodb::bson;

use crate::mongo::filter;
use crate::mongo::mongo::MongoRepo;

use crate::error::{ApiError, LocalError};
use crate::local_error;
use rocket::serde::json::json;

// Alias for ResultwT,ApiError>
type ApiResult<T> = Result<T, ApiError>;

/// Create Any API, return Mongo Oid on success, Err(e) if exception.
/// Json Any data
pub async fn create_any(db: &MongoRepo, any: serde_json::Value) -> ApiResult<Policy> {
    let new_doc = bson::to_document(&any);

    match new_doc {
        Ok(mut record) => {
            match filter::get_date_filter(&any, "context", "requestDate") {
                Ok(r) => record.insert("requestDate", bson::DateTime::from_chrono(r)),
                Err(_e) => {
                    return Err(local_error!(
                        LocalError::ContextError,
                        "Invalid requestDate"
                    ));
                }
            };

            match filter::get_date_filter(&any, "context", "policyStartDate") {
                Ok(r) => record.insert("policyStartDate", bson::DateTime::from_chrono(r)),
                Err(_e) => {
                    return Err(local_error!(
                        LocalError::ContextError,
                        "Invalid policyStartDate"
                    ));
                }
            };

            match filter::get_date_filter(&any, "context", "policyEndDate") {
                Ok(r) => record.insert("policyEndDate", bson::DateTime::from_chrono(r)),
                Err(_e) => {
                    return Err(local_error!(
                        LocalError::ContextError,
                        "Invalid policyEndDate"
                    ));
                }
            };

            record.insert(
                "integrationDate",
                bson::DateTime::from_chrono(chrono::Utc::now()),
            );

            let r = match db.policy_col.insert_one(record, None).await {
                Ok(o) => o,
                Err(e) => return Err(local_error!(LocalError::ConnectionError, format!("{}", e))),
            };

            let response = match bson::from_bson(r.inserted_id) {
                Ok(o) => o,
                Err(_e) => {
                    return Err(local_error!(
                        LocalError::ParsingError,
                        "Output parsing result exception."
                    ));
                }
            };

            let policy = Policy {
                id: None,
                content: response,
            };

            return Ok(policy);
        }
        Err(_e) => {
            return Err(local_error!(
                LocalError::ParsingError,
                "Document creation exception"
            ))
        }
    }
}

/// Return single Any from a get Oid
//pub async fn get_any(db : &MongoRepo, id : &String) -> Result<Policy,std::io::Error> {
pub async fn get_any(db: &MongoRepo, id: &String) -> ApiResult<Policy> {
    let obj_id = match ObjectId::parse_str(id) {
        //.ok().expect("Error parsing object Id");
        Ok(obj) => obj,
        Err(_e) => {
            return Err(local_error!(
                LocalError::OidFormatError,
                "ObjectId wrongly structure."
            ));
        }
    };

    let filter = doc!("_id": obj_id);

    let record = match db.policy_col.find_one(filter, None).await {
        Ok(o) => o,
        Err(e) => {
            return Err(local_error!(
                LocalError::ConnectionError,
                format!("Exception on get any : {}", e)
            ));
        }
    };

    let docfound = match record {
        Some(d) => d,
        None => {
            return Err(local_error!(LocalError::DataNotFoundError, "No result."));
        }
    };

    let response = match bson::from_bson(bson::Bson::Document(docfound)) {
        Ok(o) => o,
        Err(_e) => {
            return Err(local_error!(
                LocalError::ParsingError,
                "Output parsing result exception."
            ));
        }
    };

    let policy = Policy {
        id: None,
        content: response,
    };

    Ok(policy)
}

/// Return  all any
pub async fn get_all_any(
    db: &MongoRepo,
    filter_date: Option<String>,
    filter_policyholder: Option<String>,
    pagination: (i64, i64),
) -> ApiResult<Vec<Policy>> {
    let filter;

    if let Some(d) = filter_date {
        let rq = json!({"requestdate":d.as_str()});
        // ? for error propagating link to the alias
        filter = filter::create_date_filter(rq)?;
    } else {
        if let Some(p) = filter_policyholder {
            let rq = json!({"policy.name":p.as_str()});
            // ? for error propagating link to the alias
            filter = filter::create_string_filter(rq)?;
        } else {
            let error = local_error!(LocalError::ParsingError, "No filter defined.");
            return Err(error);
        }
    }

    let (page, limit) = pagination;

    // filter return object from MongoDB
    let cursors = match db
        .policy_col
        .find(
            Some(filter),
            Some(filter::head_filter(page, limit).unwrap()),
        )
        .await
    {
        Ok(o) => o,
        Err(_e) => {
            eprintln!("Exception while reading all data from filter : {}", _e);
            let error = local_error!(LocalError::ParsingError, "OKOK");
            return Err(error);
        }
    };

    //let result = bson::to_json(&record);
    let record: Vec<Policy> = cursors
        .map(|doc| Policy {
            id: None,
            content: bson::from_bson(bson::Bson::Document(doc.unwrap())).unwrap(),
        })
        .collect()
        .await;
    Ok(record)
}

/// Return  all any
pub async fn count_all_any(
    db: &MongoRepo,
    filter_date: Option<String>,
    filter_policyholder: Option<String>,
) -> ApiResult<u64> {
    let filter;

    if let Some(d) = filter_date {
        let rq = json!({"requestdate":d.as_str()});
        // ? for error propagating link to the alias
        filter = filter::create_date_filter(rq)?;
    } else {
        if let Some(p) = filter_policyholder {
            let rq = json!({"policy.name":p.as_str()});
            // ? for error propagating link to the alias
            filter = filter::create_string_filter(rq)?;
        } else {
            let error = local_error!(LocalError::ParsingError, "No filter defined.");
            return Err(error);
        }
    }

    // filter return object from MongoDB
    let cursors = match db.policy_col.count_documents(Some(filter), None).await {
        Ok(o) => o,
        Err(_e) => {
            eprintln!("Exception while reading count data from filter : {}", _e);
            let error = local_error!(LocalError::ParsingError, "Output parsing failed.");
            return Err(error);
        }
    };

    Ok(cursors)
}

/// Delete Any based on an Oid
/// first Get Any Raw Data, Stored it in Deleted collection, then remove from Policy collection
pub async fn delete_any(db: &MongoRepo, id: &String) -> ApiResult<DeleteResult> {
    let obj_id = match ObjectId::parse_str(id) {
        //.ok().expect("Error parsing object Id");
        Ok(obj) => obj,
        Err(_e) => {
            return Err(local_error!(LocalError::OidFormatError, "Wrong structure."));
        }
    };

    let filter = doc!("_id": obj_id);

    // get policy from policy collection
    match get_any(db, id).await {
        Ok(r) => {
            let new_doc = bson::to_document(&r);

            match new_doc {
                Ok(record) => {
                    let _record = match db.deleted_col.insert_one(record, None).await {
                        Ok(o) => o,
                        Err(e) => {
                            return Err(local_error!(
                                LocalError::ConnectionError,
                                format!("Insert in delete store failed : {}", e)
                            ));
                        }
                    };
                }
                Err(_e) => {
                    return Err(local_error!(
                        LocalError::ParsingError,
                        "Get Api from Oid response failed to be parsed."
                    ));
                }
            }
        }
        Err(e) => {
            return Err(e);
        }
    }

    let policy_detail = match db.policy_col.delete_one(filter, None).await {
        Ok(o) => o,
        Err(e) => {
            return Err(local_error!(
                LocalError::ConnectionError,
                format!("Delete from policy store failed: {}", e)
            ));
        }
    };

    Ok(policy_detail)
}

/// Delete Any based on an Oid
/// first Get Any Raw Data,Create new one after attaching old ObjectId, Stored it in History collection, then remove original from Policy collection
pub async fn update_any(db: &MongoRepo, any: serde_json::Value, id: String) -> ApiResult<Policy> {
    let obj_id = match ObjectId::parse_str(&id) {
        Ok(obj) => obj,
        Err(_e) => {
            return Err(local_error!(LocalError::OidFormatError, "Wrong structure."));
        }
    };

    let filter = doc!("_id": obj_id);

    // get policy from policy collection
    match get_any(db, &id).await {
        Ok(r) => {
            let mut data = any.clone();

            // retrieve existing objectId from Get and push the new one in the Body to list in array of previous modification
            let v = r.content["previousObjectIds"].clone();

            data["previousObjectIds"] = match v.as_array() {
                Some(o) => {
                    let mut vec = Vec::new();
                    vec.push(id);
                    for item in o {
                        vec.push(item.as_str().unwrap().to_string());
                    }
                    json!(vec)
                }
                None => {
                    //let arr: [String; 1] = [id];
                    let mut vec = Vec::new();
                    vec.push(id);
                    json!(vec)
                }
            };

            match create_any(db, data).await {
                Ok(created_policy) => {
                    let new_doc = bson::to_document(&r.content);

                    match new_doc {
                        Ok(record) => {
                            // History older version
                            let _rr = match db.history_col.insert_one(&record, None).await {
                                Ok(o) => o,
                                Err(_e) => {
                                    return Err(local_error!(
                                        LocalError::ConnectionError,
                                        "Insert in history failed."
                                    ));
                                }
                            };

                            // Delete current version
                            let _policy_deleted = match db.policy_col.delete_one(filter, None).await
                            {
                                Ok(o) => o,
                                Err(_e) => {
                                    return Err(local_error!(
                                        LocalError::ConnectionError,
                                        "Delete from policy failed."
                                    ));
                                }
                            };

                            return Ok(created_policy);
                        }
                        Err(_e) => {
                            return Err(local_error!(
                                LocalError::ParsingError,
                                "Parsing create output failed."
                            ));
                        }
                    }
                }
                Err(e) => {
                    return Err(local_error!(
                        LocalError::ConnectionError,
                        format!("Create policy failed : {}.", e)
                    ));
                }
            };
        }
        Err(e) => {
            return Err(local_error!(
                LocalError::ConnectionError,
                format!("Get policy failed: {}.", e)
            ));
        }
    }
}
