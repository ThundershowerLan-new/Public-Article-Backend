use crate::utils::*;
use actix_web::{Error, HttpRequest, HttpResponse, cookie::Cookie, error, put, web};
use libsql::{Database, params};
use serde_json::Value;

#[put("/{id}")]
pub(crate) async fn user(
    database: web::Data<Database>,
    json: web::Json<Value>,
    request: HttpRequest,
    path: web::Path<u64>,
) -> Result<HttpResponse, Error> {
    let connection = database
        .connect()
        .map_err(|_| error::ErrorInternalServerError("Internal Server Error"))?;
    let password = json.get_str("password")?;
    let id = path.into_inner();

    connection
        .query_row(
            "SELECT 1 FROM users WHERE id = ? AND password = ?",
            params![
                id,
                request
                    .cookie("password")
                    .ok_or(error::ErrorUnauthorized("Unauthorized"))?
                    .value()
            ],
        )
        .await?
        .ok_or(error::ErrorUnauthorized("Unauthorized"))?;

    connection
        .execute(
            "UPDATE users SET name = ?, password = ? WHERE id = ?",
            params![json.get_str("name")?, password, id],
        )
        .await
        .map_err(|_| error::ErrorInternalServerError("Internal Server Error"))?;

    Ok(HttpResponse::NoContent()
        .cookie(
            Cookie::build("password", password)
                .path("/")
                .http_only(true)
                .same_site(SameSite::None)
                .secure(true)
                .finish(),
        )
        .finish())
}
#[put("/{id}")]
pub(crate) async fn article(
    database: web::Data<Database>,
    json: web::Json<Value>,
    request: HttpRequest,
    path: web::Path<u64>,
) -> Result<HttpResponse, Error> {
    let connection = database
        .connect()
        .map_err(|_| error::ErrorInternalServerError("Internal Server Error"))?;
    let id = path.into_inner();
    let creator = connection
        .query_row("SELECT creator FROM articles WHERE id = ?", params![id])
        .await?
        .ok_or(error::ErrorNotFound("Not Found"))?
        .get_u64(0)?;

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

    connection
        .execute(
            "UPDATE articles SET title = ?, body = ? WHERE id = ?",
            params![json.get_str("title")?, json.get_str("body")?, id],
        )
        .await
        .map_err(|_| error::ErrorInternalServerError("Internal Server Error"))?;

    Ok(HttpResponse::NoContent().finish())
}
