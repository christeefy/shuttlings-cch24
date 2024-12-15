use std::sync::Arc;

use axum::{
    extract::{rejection::JsonRejection, State},
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BucketUnit {
    Liters(f32),
    Gallons(f32),
    Litres(f32),
    Pints(f32),
}

impl BucketUnit {
    fn convert(self) -> Self {
        const LITERS_TO_GALLONS: f32 = 3.7854125;
        const LITRES_TO_PINTS: f32 = 1.759754;
        match self {
            Self::Liters(amount) => Self::Gallons(amount / LITERS_TO_GALLONS),
            Self::Gallons(amount) => Self::Liters(amount * LITERS_TO_GALLONS),
            Self::Pints(amount) => Self::Litres(amount / LITRES_TO_PINTS),
            Self::Litres(amount) => Self::Pints(amount * LITRES_TO_PINTS),
        }
    }
}

pub async fn milk(
    State(state): State<AppState>,
    headers: HeaderMap,
    payload: Result<Json<BucketUnit>, JsonRejection>,
) -> impl IntoResponse {
    if !state.read().await.rate_limiter.try_acquire(1) {
        return (StatusCode::TOO_MANY_REQUESTS, "No milk available\n").into_response();
    }
    if !headers.contains_key(CONTENT_TYPE) || headers[CONTENT_TYPE] != "application/json" {
        return (StatusCode::OK, "Milk withdrawn\n").into_response();
    }
    match payload {
        Ok(Json(bucket_unit)) => (
            StatusCode::OK,
            serde_json::to_string(&bucket_unit.convert()).unwrap(),
        )
            .into_response(),
        Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}

pub async fn refill(State(state): State<AppState>) -> impl IntoResponse {
    state.write().await.reload_rate_limiter();
    StatusCode::OK
}
