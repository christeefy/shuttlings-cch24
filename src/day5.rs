use axum::http::StatusCode;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Package,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    authors: Vec<String>,
    keywords: Vec<String>,
    metadata: Metadata,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    orders: Vec<Order>,
}

#[derive(Debug, Deserialize)]
struct Order {
    item: String,
    quantity: u32,
}

pub async fn manifest(body: String) -> Result<String, StatusCode> {
    let payload: CargoToml = toml::from_str(&body).map_err(|_| StatusCode::NO_CONTENT)?;
    let summary = payload
        .package
        .metadata
        .orders
        .iter()
        .map(|order| format!("{}: {}", order.item, order.quantity))
        .collect::<Vec<_>>()
        .join("\n");
    dbg!(&payload);
    Ok(summary)
}
