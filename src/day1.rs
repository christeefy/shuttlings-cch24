use axum::{body::Body, http::header, response::Response};

pub async fn hello_world() -> &'static str {
    "Hello, bird!"
}

pub async fn seek() -> Response {
    Response::builder()
        .status(302)
        .header(
            header::LOCATION,
            "https://www.youtube.com/watch?v=9Gc4QTqslN4",
        )
        .body(Body::empty())
        .unwrap()
}
