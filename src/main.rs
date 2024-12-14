mod day1;
mod day2;
mod day5;

use axum::{
    routing::{get, post},
    Router,
};

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(day1::hello_world))
        .route("/-1/seek", get(day1::seek))
        .route("/2/dest", get(day2::dest))
        .route("/2/key", get(day2::key))
        .route("/2/v6/dest", get(day2::dest_v6))
        .route("/2/v6/key", get(day2::key_v6))
        .route("/5/manifest", post(day5::manifest));
    Ok(router.into())
}
