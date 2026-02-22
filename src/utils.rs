use actix_web::{
    Error,
    error::{ErrorBadRequest, ErrorInternalServerError},
    web::Data,
};
use libsql::{Builder, Connection, Database, Row, params::IntoParams};
use serde_json::{Value, json};
use std::env;

pub(crate) async fn initialize_database() -> Data<Database> {
    Data::new(
        Builder::new_remote(
            env::var("URL").expect("Failed to get URL"),
            env::var("TOKEN").expect("Failed to get TOKEN"),
        )
        .build()
        .await
        .expect("Failed to initialize database"),
    )
}

pub(crate) trait Query {
    async fn query_row(&self, sql: &str, params: impl IntoParams) -> Result<Option<Row>, Error>;
    async fn query_articles(&self, sql: &str, params: impl IntoParams)
    -> Result<Vec<Value>, Error>;
}

impl Query for Connection {
    async fn query_row(&self, sql: &str, params: impl IntoParams) -> Result<Option<Row>, Error> {
        self.query(sql, params)
            .await
            .map_err(|_| ErrorInternalServerError("Internal Server Error"))?
            .next()
            .await
            .map_err(|_| ErrorInternalServerError("Internal Server Error"))
    }

    async fn query_articles(
        &self,
        sql: &str,
        params: impl IntoParams,
    ) -> Result<Vec<Value>, Error> {
        let mut articles = Vec::new();
        let mut rows = self
            .query(sql, params)
            .await
            .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;

        while let Some(row) = rows
            .next()
            .await
            .map_err(|_| ErrorInternalServerError("Internal Server Error"))?
        {
            articles.push(json!({
                "id": row.get_u64(0)?,
                "title": row.get_string(1)?
            }));
        }

        Ok(articles)
    }
}

pub(crate) trait RowGet {
    fn get_u64(&self, idx: i32) -> Result<u64, Error>;
    fn get_string(&self, idx: i32) -> Result<String, Error>;
}

impl RowGet for Row {
    fn get_u64(&self, idx: i32) -> Result<u64, Error> {
        self.get::<u64>(idx)
            .map_err(|_| ErrorInternalServerError("Internal Server Error"))
    }

    fn get_string(&self, idx: i32) -> Result<String, Error> {
        self.get::<String>(idx)
            .map_err(|_| ErrorInternalServerError("Internal Server Error"))
    }
}

pub(crate) trait ValueGet {
    fn get_u64(&self, idx: &str) -> Result<u64, Error>;
    fn get_str(&self, idx: &str) -> Result<&str, Error>;
}

impl ValueGet for Value {
    fn get_u64(&self, idx: &str) -> Result<u64, Error> {
        self.get(idx)
            .and_then(|value| value.as_u64())
            .ok_or(ErrorBadRequest("Bad Request"))
    }

    fn get_str(&self, idx: &str) -> Result<&str, Error> {
        self.get(idx)
            .and_then(|value| value.as_str())
            .ok_or(ErrorBadRequest("Bad Request"))
    }
}
