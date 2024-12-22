use axum::{
    extract::{rejection::JsonRejection, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sqlx::types::{
    chrono::{DateTime, Utc},
    Uuid,
};

use crate::AppState;

pub async fn reset(State(state): State<AppState>) -> impl IntoResponse {
    const TABLE_NAME: &str = "quotes";

    let pool = &state.read().await.pool;

    match sqlx::query(&format!("TRUNCATE TABLE {TABLE_NAME}"))
        .execute(pool)
        .await
    {
        Ok(_) => (StatusCode::OK, "Quotes table has been reset".to_string()),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e:?}")),
    }
}

pub async fn cite(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let pool = &state.read().await.pool;
    match sqlx::query_as!(Quote, "SELECT * FROM quotes WHERE id = ($1)", id)
        .fetch_one(pool)
        .await
    {
        Ok(quote) => (StatusCode::OK, serde_json::to_string(&quote).unwrap()),
        Err(sqlx::Error::RowNotFound) => (
            StatusCode::NOT_FOUND,
            format!("Quote ID {id} does not exist"),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch data from DB".to_string(),
        ),
    }
}

pub async fn remove(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let pool = &state.read().await.pool;
    match sqlx::query_as!(Quote, "DELETE FROM quotes WHERE id = ($1) RETURNING *", id)
        .fetch_one(pool)
        .await
    {
        Ok(quote) => (StatusCode::OK, serde_json::to_string(&quote).unwrap()),
        Err(sqlx::Error::RowNotFound) => (
            StatusCode::NOT_FOUND,
            format!("Quote ID {id} does not exist"),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch data from DB".to_string(),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct QuotePayload {
    author: String,
    quote: String,
}

pub async fn undo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    payload: Result<Json<QuotePayload>, JsonRejection>,
) -> impl IntoResponse {
    let Json(payload) = payload.unwrap();
    let pool = &state.read().await.pool;

    match sqlx::query_as!(Quote,
        "UPDATE quotes SET author = ($1), quote = ($2), version = version + 1 WHERE id = ($3) RETURNING *",
        payload.author,
        payload.quote,
        id,
    )
    .fetch_one(pool)
    .await
    {
        Ok(quote) => (StatusCode::OK, serde_json::to_string(&quote).unwrap()),
        Err(sqlx::Error::RowNotFound) => (
            StatusCode::NOT_FOUND,
            format!("Quote ID {id} does not exist"),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch data from DB".to_string(),
        ),
    }
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Quote {
    id: Uuid,
    author: String,
    quote: String,
    created_at: DateTime<Utc>,
    version: i32,
}

impl Quote {
    pub fn new(author: String, quote: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            author,
            quote,
            created_at: Utc::now(),
            version: 1,
        }
    }
}

pub async fn draft(
    State(state): State<AppState>,
    payload: Result<Json<QuotePayload>, JsonRejection>,
) -> impl IntoResponse {
    let Json(payload) = payload.unwrap();
    let pool = &state.read().await.pool;
    let quote = Quote::new(payload.author.clone(), payload.quote.clone()); // TODO: Remove clone
    match sqlx::query!(
        "INSERT INTO quotes (id, author, quote, created_at, version) VALUES ($1, $2, $3, $4, $5)",
        quote.id,
        quote.author,
        quote.quote,
        quote.created_at,
        quote.version
    )
    .execute(pool)
    .await
    {
        Ok(_) => (StatusCode::CREATED, serde_json::to_string(&quote).unwrap()),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to draft quote".to_string(),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListParam {
    token: String,
}

#[derive(Debug, Serialize)]
pub struct ListResponse<'a> {
    quotes: &'a [Quote],
    page: u32,
    next_token: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    query: Option<Query<ListParam>>,
) -> impl IntoResponse {
    const PAGE_SIZE: usize = 3; // sqlx integers cannot be unsigned
    const TOKEN_LENGTH: usize = 16;

    let page_number = match query {
        Some(Query(ListParam { token })) => match state.write().await.list_tokens.remove(&token) {
            Some(page_number) => page_number,
            None => return (StatusCode::BAD_REQUEST, "Invalid token".to_string()),
        },
        None => 1,
    };
    let page_offset = (page_number - 1) * 3;

    let Ok(quotes) = sqlx::query_as!(
        Quote,
        "SELECT *
            FROM quotes
            ORDER BY
                created_at
            LIMIT ($1)
            OFFSET ($2)",
        (PAGE_SIZE + 1) as i64,
        page_offset as i64
    )
    .fetch_all(&state.read().await.pool)
    .await
    else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Query failed".to_string(),
        );
    };

    let next_token = if quotes.len() > PAGE_SIZE {
        Some(
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(TOKEN_LENGTH)
                .map(char::from)
                .collect(),
        )
    } else {
        None
    };

    let response = ListResponse {
        quotes: &quotes[..PAGE_SIZE.min(quotes.len())],
        page: page_number,
        next_token: next_token.clone(),
    };

    if let Ok(json_str) = serde_json::to_string(&response) {
        // When there are no more possible errors,
        // update the app state with the new token before returning
        if let Some(next_token) = next_token {
            state
                .write()
                .await
                .list_tokens
                .insert(next_token, page_number + 1);
        }
        (StatusCode::OK, json_str)
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to serialize response".to_string(),
        )
    }
}
