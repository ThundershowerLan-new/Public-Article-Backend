use crate::utils::*;
use actix_web::{Error, HttpRequest, HttpResponse, delete, error, web};
use libsql::{Database, params};

#[delete("/{id}")]
pub(crate) async fn user(
    database: web::Data<Database>,
    request: HttpRequest,
    path: web::Path<u64>,
) -> Result<HttpResponse, Error> {
    let connection = database
        .connect()
        .map_err(|_| error::ErrorInternalServerError("Internal Server Error"))?;
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
        .execute("DELETE FROM users WHERE id = ?", params![id])
        .await
        .map_err(|_| error::ErrorInternalServerError("Internal Server Error"))?;

    Ok(HttpResponse::NoContent().finish())
}

#[delete("/{id}")]
pub(crate) async fn article(
    database: web::Data<Database>,
    request: HttpRequest,
    path: web::Path<u64>,
) -> Result<HttpResponse, Error> {
    let connection = database
        .connect()
        .map_err(|_| error::ErrorInternalServerError("Internal Server Error"))?;
    let id = path.into_inner();

    connection
        .query_row(
            "SELECT 1 FROM users WHERE id = ? AND password = ?",
            params![
                connection
                    .query_row("SELECT creator FROM articles WHERE id = ?", params![id])
                    .await?
                    .ok_or(error::ErrorNotFound("Not Found"))?
                    .get_u64(0)?,
                request
                    .cookie("password")
                    .ok_or(error::ErrorUnauthorized("Unauthorized"))?
                    .value()
            ],
        )
        .await?
        .ok_or(error::ErrorUnauthorized("Unauthorized"))?;

    connection
        .execute("DELETE FROM articles WHERE id = ?", params![id])
        .await
        .map_err(|_| error::ErrorInternalServerError("Internal Server Error"))?;

    Ok(HttpResponse::NoContent().finish())
}
