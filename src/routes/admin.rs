use crate::db::{get_collection, File};
use crate::util::variables::ADMIN_TOKEN;

use actix_web::{web, HttpRequest, HttpResponse};
use futures::StreamExt;
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ListQuery {
    pub tag: Option<String>,
    pub limit: Option<i64>,
    pub skip: Option<u64>,
    pub search: Option<String>,
}

pub async fn list_files(req: HttpRequest, query: web::Query<ListQuery>) -> HttpResponse {
    // Auth — X-Bot-Token must match ADMIN_TOKEN if set
    if !ADMIN_TOKEN.is_empty() {
        let provided = req
            .headers()
            .get("X-Bot-Token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if provided != *ADMIN_TOKEN {
            return HttpResponse::Unauthorized().finish();
        }
    }

    let col = get_collection("attachments");

    let mut filter = doc! { "deleted": { "$ne": true } };

    if let Some(tag) = &query.tag {
        if !tag.is_empty() {
            filter.insert("tag", tag.as_str());
        }
    }

    if let Some(search) = &query.search {
        if !search.is_empty() {
            filter.insert("filename", doc! { "$regex": search.as_str(), "$options": "i" });
        }
    }

    let options = FindOptions::builder()
        .sort(doc! { "_id": -1 })
        .limit(query.limit.unwrap_or(100))
        .skip(query.skip.unwrap_or(0))
        .build();

    let cursor = match col.find(filter, options).await {
        Ok(c) => c,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let files: Vec<File> = cursor
        .filter_map(|r| async move { r.ok() })
        .collect()
        .await;

    HttpResponse::Ok().json(files)
}
