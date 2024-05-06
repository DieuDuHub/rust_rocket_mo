use chrono::prelude::*;

use crate::error::{ApiError, LocalError};
use crate::local_error;
use bson::{Bson, Document};
use mongodb::options::FindOptions;

pub fn get_date_filter(
    v: &serde_json::Value,
    root: &str,
    sub: &str,
) -> Result<DateTime<Utc>, ApiError> {
    //let new_doc = bson::Document();
    let _request_date = match v[root][sub].as_str() {
        Some(o) => {
            match o.parse::<DateTime<Utc>>() {
                Ok(oo) => return Ok(oo),
                Err(_e) => {
                    return Err(local_error!(
                        LocalError::ContextError,
                        format!("Exception on value {} {}", root, sub)
                    ));
                }
            };
        }
        None => {
            return Err(local_error!(
                LocalError::ContextError,
                format!("Exception on value {} {}", root, sub)
            ));
        }
    };
}

pub fn create_date_filter(filter: serde_json::Value) -> Result<Document, ApiError> {
    let mut doc: Document = Document::new();

    let requestdate = match filter["requestdate"].as_str() {
        Some(o) => o.parse::<DateTime<Utc>>(),
        None => {
            return Err(local_error!(
                LocalError::FilterDateParsing,
                format!("Filter value not found requestDate")
            ));
        }
    };

    match requestdate {
        Ok(o) => {
            doc.insert("$gte", bson::DateTime::from_chrono(o));

            let mut document: Document = Document::new();

            document.insert("requestDate", Bson::Document(doc));

            Ok(document)
        }
        Err(_e) => {
            return Err(local_error!(
                LocalError::FilterStringarsing,
                "Date filter wrongly formatted."
            ));
        }
    }
}

pub fn create_string_filter(filter: serde_json::Value) -> Result<Document, ApiError> {
    let bson_req = match bson::to_bson(&filter) {
        Ok(o) => o,
        Err(_e) => {
            return Err(local_error!(
                LocalError::ContextError,
                "Json String filter to bson wrongly structure."
            ))
        }
    };

    if let bson::Bson::Document(document) = bson_req {
        return Ok(document);
    } else {
        return Err(local_error!(
            LocalError::FilterDateParsing,
            "Input Json String filter Data wrongly structure."
        ));
    }
}

pub fn head_filter(page: i64, limit: i64) -> Result<FindOptions, ApiError> {
    let mut doc = Document::new();

    let search_policies_filter = [
        "policy.location",
        "policy.title",
        "policy.toto",
        "requestDate",
        "policyStartDate",
        "policyEndDate",
    ];

    for v in &search_policies_filter {
        doc.insert(v.to_string(), Bson::Int32(0));
    }

    let find_options = FindOptions::builder()
        .limit(limit)
        .skip(u64::try_from((page - 1) * limit).unwrap())
        .projection(doc)
        .build();

    Ok(find_options)
}
