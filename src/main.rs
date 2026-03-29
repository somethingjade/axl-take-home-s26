use reqwest::header;
use reqwest::Method;
use tower_http::cors::Any;
use axum::{Json, Router, extract::State, routing::post};
use serde::{Deserialize, Serialize};
use server::{game, init, state::AppState};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[derive(Deserialize)]
struct Input {
    message: String,
}

#[derive(Serialize)]
struct Output {
    reply: String,
}

async fn handler(State(state): State<AppState>, Json(input): Json<Input>) -> Json<Output> {
    let mut inner_state = state.state.lock().await;
    let reply = game::run(&mut inner_state, &input.message).await;
    Json(Output { reply })
}

#[tokio::main]
async fn main() {
    let state = init::init();
    let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
    .allow_headers([header::CONTENT_TYPE]);
    let app = Router::new()
        .route("/play", post(handler).options(async || {}))
        .with_state(state)
        .layer(cors);
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap();
    let addr = format!("0.0.0.0:{}", port);
    println!("Running on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
