use crate::utils::*;
use actix_web::{Error, HttpResponse, error, get, web};
use libsql::{Database, params};
use serde_json::json;

#[get("")]
pub(crate) async fn index(database: web::Data<Database>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(json!({
        "articles": database
            .connect()
            .map_err(|_| error::ErrorInternalServerError("Internal Server Error"))?
            .query_articles("SELECT id, title FROM articles WHERE id > 10", params![])
            .await?
    })))
}

#[get("/{id}")]
pub(crate) async fn user(
    database: web::Data<Database>,
    path: web::Path<u64>,
) -> Result<HttpResponse, Error> {
    let connection = database
        .connect()
        .map_err(|err| error::ErrorInternalServerError(err))?;
    let id = path.into_inner();

    Ok(HttpResponse::Ok().json(json!({
        "name": connection
            .query_row("SELECT name FROM users WHERE id = ?", params![id])
            .await?
            .ok_or(error::ErrorNotFound("Not Found"))?
            .get_string(0)?,
        "articles": connection
            .query_articles(
                "SELECT id, title FROM articles WHERE creator = ?",
                params![id],
            ).await?
    })))
}

#[get("/{id}")]
pub(crate) async fn article(
    database: web::Data<Database>,
    path: web::Path<u64>,
) -> Result<HttpResponse, Error> {
    let row = database
        .connect()
        .map_err(|_| error::ErrorInternalServerError("Internal Server Error"))?
        .query_row(
            "SELECT title, body, creator FROM articles WHERE id = ?",
            params![path.into_inner()],
        )
        .await?
        .ok_or(error::ErrorNotFound("Not Found"))?;

    Ok(HttpResponse::Ok().json(json!({
        "title": row.get_string(0)?,
        "body": row.get_string(1)?,
        "creator": row.get_u64(2)?
    })))
}
