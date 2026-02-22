use crate::utils::*;
use actix_web::{Error, HttpRequest, HttpResponse, cookie::Cookie, error, post, web};
use libsql::{Database, params};
use serde_json::{Value, json};

#[post("")]
pub(crate) async fn index(
    database: web::Data<Database>,
    json: web::Json<Value>,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(json!({
        "articles": database
            .connect()
            .map_err(|_| error::ErrorInternalServerError("Internal Server Error"))?
            .query_articles(
                "SELECT rowid, title FROM articles_fts WHERE articles_fts MATCH ?",
                params![json.get_str("words")?],
            ).await?
    })))
}

#[post("")]
pub(crate) async fn user(
    database: web::Data<Database>,
    json: web::Json<Value>,
) -> Result<HttpResponse, Error> {
    let connection = database
        .connect()
        .map_err(|_| error::ErrorInternalServerError("Internal Server Error"))?;
    let name = json.get_str("name")?;
    let password = json.get_str("password")?;

    match connection
        .query_row(
            "INSERT INTO users (name, password) VALUES (?, ?) ON CONFLICT(name) DO NOTHING RETURNING id",
            params![name, password],
        ).await? {

        Some(row) => Ok(HttpResponse::Created()
            .cookie(
                Cookie::build("password", password)
                    .path("/")
                    .http_only(true)
                    .finish()
            ).json(json!({
            "id": row.get_u64(0)?
        }))),
        None => {
            let row = connection
                .query_row(
                    "SELECT id, password FROM users WHERE name = ?",
                    params![name]
                ).await?
                .ok_or(error::ErrorNotFound("Not Found"))?;

            if password == row.get_string(1)? {
                Ok(HttpResponse::Ok()
                    .cookie(
                        Cookie::build("password", password)
                            .path("/")
                            .http_only(true)
                            .finish()
                    ).json(json!({
                        "id": row.get_u64(0)?
                    }))
                )
            } else {
                Err(error::ErrorUnauthorized("Unauthorized"))
            }
        }
    }
}

#[post("")]
pub(crate) async fn article(
    database: web::Data<Database>,
    json: web::Json<Value>,
    request: HttpRequest,
) -> Result<HttpResponse, Error> {
    let connection = database
        .connect()
        .map_err(|_| error::ErrorInternalServerError("Internal Server Error"))?;
    let creator = json.get_u64("creator")?;

    connection
        .query_row(
            "SELECT 1 FROM users WHERE id = ? AND password = ?",
            params![
                creator,
                request
                    .cookie("password")
                    .ok_or(error::ErrorUnauthorized("Unauthorized"))?
                    .value()
            ],
        )
        .await?
        .ok_or(error::ErrorUnauthorized("Unauthorized"))?;

    Ok(HttpResponse::Created().json(json!({
        "id": connection.query_row("INSERT INTO articles (title, body, creator) VALUES (?, ?, ?) RETURNING id", params![
            json.get_str("title")?,
            json.get_str("body")?,
            creator
        ]).await?
        .ok_or(error::ErrorInternalServerError("Internal Server Error"))?
        .get_u64(0)?
    })))
}
