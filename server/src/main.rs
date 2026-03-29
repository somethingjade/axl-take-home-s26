use axum::{Json, Router, extract::State, routing::post};
use serde::{Deserialize, Serialize};
use server::{game::{self, GameState}, init, state::AppState};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

#[derive(Deserialize)]
struct Input {
    session_id: Uuid,
    message: String,
}

#[derive(Serialize)]
struct Output {
    reply: String,
}

#[derive(Serialize)]
struct SessionResponse {
    session_id: String,
}

async fn new_session(
    State(state): State<AppState>,
) -> Json<SessionResponse> {
    let session_id = Uuid::new_v4();
    let session_id_string = session_id.to_string();
    let mut inner_state = state.state.lock().await;
    let sessions = &mut inner_state.sessions;
    sessions.insert(session_id.clone(), GameState::init());
    Json(SessionResponse { session_id: session_id_string })
}

async fn handler(State(state): State<AppState>, Json(input): Json<Input>) -> Json<Output> {
    println!("[DEBUG] Input: {}", input.message);
    let mut inner_state = state.state.lock().await;
    let reply = game::run(&mut inner_state, &input.session_id, &input.message).await;
    println!("[DEBUG] Reply: {}", reply);
    Json(Output { reply })
}

#[tokio::main]
async fn main() {
    let state = init::init();
    let cors = CorsLayer::permissive();
    let app = Router::new()
        .route("/new-session", post(new_session))
        .route("/play", post(handler))
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
