mod day00;
mod day02;
mod day05;
mod day09;

use std::{sync::Arc, time::Duration};

use axum::{
    routing::{get, post},
    Router,
};
use leaky_bucket::RateLimiter;
use tokio::sync::RwLock;

type AppState = Arc<RwLock<InnerAppState>>;

struct InnerAppState {
    rate_limiter: RateLimiter,
}

impl InnerAppState {
    fn new() -> Self {
        Self {
            rate_limiter: Self::default_rate_limiter(),
        }
    }

    fn reload_rate_limiter(&mut self) {
        self.rate_limiter = Self::default_rate_limiter()
    }

    fn default_rate_limiter() -> RateLimiter {
        RateLimiter::builder()
            .initial(5)
            .max(5)
            .interval(Duration::from_secs(1))
            .build()
    }
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let state = Arc::new(RwLock::new(InnerAppState::new()));
    let router = Router::new()
        .route("/", get(day00::hello_world))
        .route("/-1/seek", get(day00::seek))
        .route("/2/dest", get(day02::dest))
        .route("/2/key", get(day02::key))
        .route("/2/v6/dest", get(day02::dest_v6))
        .route("/2/v6/key", get(day02::key_v6))
        .route("/5/manifest", post(day05::manifest))
        .route("/9/milk", post(day09::milk))
        .route("/9/refill", post(day09::refill))
        .with_state(state);
    Ok(router.into())
}
