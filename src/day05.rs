use std::str::FromStr;

use axum::{
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    response::IntoResponse,
};
use cargo_manifest::{Manifest, MaybeInherited};

use toml::Value;

fn _validate_manifest<T>(manifest: Manifest<T, T>) -> Result<T, (StatusCode, &'static str)> {
    let Some(package) = manifest.package else {
        return Err((StatusCode::NO_CONTENT, "Empty package"));
    };
    if let Some(MaybeInherited::Local(keywords)) = package.keywords {
        if !keywords.contains(&"Christmas 2024".to_string()) {
            return Err((StatusCode::BAD_REQUEST, "Magic keyword not provided"));
        }
    } else {
        return Err((StatusCode::BAD_REQUEST, "Magic keyword not provided"));
    }
    let Some(metadata) = package.metadata else {
        return Err((StatusCode::NO_CONTENT, "Empty metadata"));
    };
    Ok(metadata)
}

fn process_toml(body: String) -> (StatusCode, String) {
    let Ok(manifest) = Manifest::from_str(&body) else {
        return (StatusCode::BAD_REQUEST, "Invalid manifest".to_string());
    };

    let metadata = match _validate_manifest(manifest) {
        Ok(metadata) => metadata,
        Err((status_code, static_str)) => return (status_code, static_str.to_string()),
    };

    let maybe_orders = match metadata.get("orders") {
        Some(Value::Array(maybe_orders)) => maybe_orders,
        Some(_) => return (StatusCode::BAD_REQUEST, "Invalid metadata".to_string()),
        None => return (StatusCode::NO_CONTENT, "No orders".to_string()),
    };

    let summary = maybe_orders
        .iter()
        .filter_map(|value| match (value.get("item"), value.get("quantity")) {
            (Some(Value::String(item)), Some(Value::Integer(quantity))) => {
                Some(format!("{item}: {quantity}"))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    if summary.is_empty() {
        (StatusCode::NO_CONTENT, "No valid orders".to_string())
    } else {
        (StatusCode::OK, summary.join("\n").to_string())
    }
}

fn process_json(body: String) -> (StatusCode, String) {
    let Ok(manifest) =
        serde_json::from_str::<Manifest<serde_json::Value, serde_json::Value>>(&body)
    else {
        return (StatusCode::BAD_REQUEST, "Invalid manifest".to_string());
    };
    let metadata = match _validate_manifest(manifest) {
        Ok(metadata) => metadata,
        Err((status_code, static_str)) => return (status_code, static_str.to_string()),
    };

    let maybe_orders = match metadata.get("orders") {
        Some(serde_json::Value::Array(maybe_orders)) => maybe_orders,
        Some(_) => return (StatusCode::BAD_REQUEST, "Invalid metadata".to_string()),
        None => return (StatusCode::NO_CONTENT, "No orders".to_string()),
    };
    let summary = maybe_orders
        .iter()
        .filter_map(|value| match (value.get("item"), value.get("quantity")) {
            // TODO: Handle case when number is not an int
            (Some(serde_json::Value::String(item)), Some(serde_json::Value::Number(quantity))) => {
                Some(format!("{item}: {quantity}"))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    if summary.is_empty() {
        (StatusCode::NO_CONTENT, "No valid orders".to_string())
    } else {
        (StatusCode::OK, summary.join("\n").to_string())
    }
}

fn process_yaml(body: String) -> (StatusCode, String) {
    let Ok(manifest) =
        serde_yaml::from_str::<Manifest<serde_yaml::Value, serde_yaml::Value>>(&body)
    else {
        return (StatusCode::BAD_REQUEST, "Invalid manifest".to_string());
    };
    let metadata = match _validate_manifest(manifest) {
        Ok(metadata) => metadata,
        Err((status_code, static_str)) => return (status_code, static_str.to_string()),
    };

    let maybe_orders = match metadata.get("orders") {
        Some(serde_yaml::Value::Sequence(maybe_orders)) => maybe_orders,
        Some(_) => return (StatusCode::BAD_REQUEST, "Invalid metadata".to_string()),
        None => return (StatusCode::NO_CONTENT, "No orders".to_string()),
    };
    let summary = maybe_orders
        .iter()
        .filter_map(|value| match (value.get("item"), value.get("quantity")) {
            // TODO: Handle case when number is not an int
            (Some(serde_yaml::Value::String(item)), Some(serde_yaml::Value::Number(quantity))) => {
                Some(format!("{item}: {quantity}"))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    if summary.is_empty() {
        (StatusCode::NO_CONTENT, "No valid orders".to_string())
    } else {
        (StatusCode::OK, summary.join("\n").to_string())
    }
}

pub async fn manifest(header: HeaderMap, body: String) -> impl IntoResponse {
    match header.get(CONTENT_TYPE) {
        Some(header_value) => match header_value.to_str() {
            Ok("application/toml") => process_toml(body),
            Ok("application/json") => process_json(body),
            Ok("application/yaml") => process_yaml(body),
            _ => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Invalid content type header".to_string(),
            ),
        },
        _ => (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Invalid content type header".to_string(),
        ),
    }
}
