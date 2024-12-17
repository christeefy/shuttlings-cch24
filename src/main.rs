mod day00;
mod day02;
mod day05;
mod day09;
mod day12;
mod day16;

use std::{sync::Arc, time::Duration};

use axum::{
    routing::{get, post},
    Router,
};
use leaky_bucket::RateLimiter;
use rand::SeedableRng;
use tokio::sync::RwLock;

type AppState = Arc<RwLock<InnerAppState>>;

struct InnerAppState {
    board: day12::Board,
    rate_limiter: RateLimiter,
    rng: rand::rngs::StdRng,
    secrets: shuttle_runtime::SecretStore,
}

impl InnerAppState {
    fn new(secrets: shuttle_runtime::SecretStore) -> Self {
        Self {
            board: day12::Board::<4>::new(),
            rate_limiter: Self::default_rate_limiter(),
            rng: Self::default_rng(),
            secrets,
        }
    }

    fn reset_rng(&mut self) {
        self.rng = Self::default_rng();
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

    fn default_rng() -> rand::rngs::StdRng {
        rand::rngs::StdRng::seed_from_u64(2024)
    }
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    let state = Arc::new(RwLock::new(InnerAppState::new(secrets)));
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
        .route("/12/board", get(day12::board))
        .route("/12/random-board", get(day12::random_board))
        .route("/12/reset", post(day12::reset))
        .route("/12/place/:team/:column", post(day12::place))
        .route("/16/wrap", post(day16::wrap))
        .route("/16/unwrap", get(day16::unwrap))
        .route("/16/decode", post(day16::decode))
        .with_state(state);
    Ok(router.into())
}
