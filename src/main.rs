mod day1;
mod day2;
mod day5;
mod day9;

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
        .route("/", get(day1::hello_world))
        .route("/-1/seek", get(day1::seek))
        .route("/2/dest", get(day2::dest))
        .route("/2/key", get(day2::key))
        .route("/2/v6/dest", get(day2::dest_v6))
        .route("/2/v6/key", get(day2::key_v6))
        .route("/5/manifest", post(day5::manifest))
        .route("/9/milk", post(day9::milk))
        .route("/9/refill", post(day9::refill))
        .with_state(state);
    Ok(router.into())
}
