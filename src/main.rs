mod day00;
mod day02;
mod day05;
mod day09;
mod day12;
mod day16;
mod day19;

use std::{collections::HashMap, sync::Arc, time::Duration};

use axum::{
    routing::{delete, get, post, put},
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
    pool: sqlx::PgPool,
    list_tokens: HashMap<String, u32>,
}

impl InnerAppState {
    fn new(secrets: shuttle_runtime::SecretStore, pool: sqlx::PgPool) -> Self {
        Self {
            board: day12::Board::<4>::new(),
            rate_limiter: Self::default_rate_limiter(),
            rng: Self::default_rng(),
            secrets,
            pool,
            list_tokens: HashMap::new(),
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
    #[shuttle_shared_db::Postgres] pool: sqlx::PgPool,
) -> shuttle_axum::ShuttleAxum {
    // Stand up database
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    let state = Arc::new(RwLock::new(InnerAppState::new(secrets, pool)));
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
        .route("/19/reset", post(day19::reset))
        .route("/19/cite/:id", get(day19::cite))
        .route("/19/remove/:id", delete(day19::remove))
        .route("/19/undo/:id", put(day19::undo))
        .route("/19/draft", post(day19::draft))
        .route("/19/list", get(day19::list))
        .with_state(state);
    Ok(router.into())
}
