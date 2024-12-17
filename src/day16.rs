use std::collections::HashSet;

use axum::{
    extract::State,
    http::{
        header::{CONTENT_TYPE, COOKIE, SET_COOKIE},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::IntoResponse,
};
use jsonwebtoken::{errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use regex::Regex;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    iss: String,
    exp: i64,
    payload: serde_json::Value,
}

impl Claims {
    fn new(payload: serde_json::Value, exp: i64) -> Self {
        Self {
            sub: "gift".to_string(),
            iss: "Santa Clause".to_string(),
            exp,
            payload,
        }
    }
}

pub async fn wrap(
    State(state): State<AppState>,
    header: HeaderMap,
    body: String,
) -> impl IntoResponse {
    // Validate "Content-Type" header
    match header.get(CONTENT_TYPE) {
        Some(header_value) => match header_value.to_str() {
            Ok("application/json") => (),
            _ => {
                return (
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    "Only JSON supported".to_string(),
                )
                    .into_response();
            }
        },
        None => {
            return (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Empty content type header".to_string(),
            )
                .into_response();
        }
    };

    let Some(jwt_secret) = state.read().await.secrets.get("JWT_SECRET") else {
        return (
            StatusCode::FAILED_DEPENDENCY,
            "Failed to load secrets".to_string(),
        )
            .into_response();
    };

    let dt = OffsetDateTime::now_utc();
    let exp = (dt + Duration::days(1)).unix_timestamp();
    let payload: serde_json::Value = serde_json::from_str(&body).unwrap();
    let jwt = jsonwebtoken::encode(
        &Header::default(),
        &Claims::new(payload, exp),
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .unwrap(); // TODO

    let mut response_header = HeaderMap::new();
    response_header.insert(
        SET_COOKIE,
        HeaderValue::from_str(&format!("gift={}", jwt)).unwrap(),
    );
    (StatusCode::OK, response_header).into_response()
}

pub async fn unwrap(State(state): State<AppState>, header: HeaderMap) -> impl IntoResponse {
    let Some(header_value) = header.get(COOKIE) else {
        return (StatusCode::BAD_REQUEST, "Missing cookie".to_string());
    };
    let Ok(cookie) = header_value.to_str() else {
        return (StatusCode::BAD_REQUEST, "Invalid cookie".to_string());
    };

    let re = Regex::new(r"gift=(.*)").expect("Invalid regex provided");

    let Some(captures) = re.captures(cookie) else {
        return (StatusCode::BAD_REQUEST, "Invalid cookie".to_string());
    };
    let Some(regex_match) = captures.get(1) else {
        return (StatusCode::BAD_REQUEST, "Invalid cookie".to_string());
    };
    let jwt = regex_match.as_str();

    let Some(jwt_secret) = state.read().await.secrets.get("JWT_SECRET") else {
        return (
            StatusCode::FAILED_DEPENDENCY,
            "Failed to load secrets".to_string(),
        );
    };

    jsonwebtoken::decode::<Claims>(
        jwt,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .unwrap();

    let Ok(token_data) = jsonwebtoken::decode::<Claims>(
        jwt,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    ) else {
        return (StatusCode::BAD_REQUEST, "Invalid JWT".to_string());
    };

    let Ok(payload) = serde_json::to_string(&token_data.claims.payload) else {
        return (StatusCode::BAD_REQUEST, "Invalid JSON".to_string());
    };

    (StatusCode::OK, payload)
}

#[derive(Debug, Serialize, Deserialize)]
struct SimpleClaims {
    // sub: String,
    // iss: String,
    exp: i64,
    // payload: serde_json::Value,
}

pub async fn decode(body: String) -> impl IntoResponse {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.required_spec_claims = HashSet::new();
    validation.algorithms.push(Algorithm::RS512);

    let res = jsonwebtoken::decode::<serde_json::Value>(
        &body,
        &DecodingKey::from_rsa_pem(include_bytes!("./day16_santa_public_key.pem")).unwrap(),
        &validation,
    );

    match res {
        Ok(token_data) => (
            StatusCode::OK,
            serde_json::to_string(&token_data.claims).unwrap(),
        ),
        Err(error) => match error.kind() {
            ErrorKind::InvalidSignature => {
                (StatusCode::UNAUTHORIZED, "You're not Santa!".to_string())
            }
            _ => (
                StatusCode::BAD_REQUEST,
                format!("Failed to decode: {:?}", error.kind()),
            ),
        },
    }
}
