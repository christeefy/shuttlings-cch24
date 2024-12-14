mod day1;

use axum::{routing::get, Router};

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(day1::hello_world))
        .route("/-1/seek", get(day1::seek));
    Ok(router.into())
}
