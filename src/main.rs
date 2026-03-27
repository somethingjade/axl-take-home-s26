use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[derive(Deserialize)]
struct GameInput {
    message: String,
}

#[derive(Serialize)]
struct GameOutput {
    reply: String,
}

async fn call_llm(prompt: &str) -> String {
    let api_key = std::env::var("OPENAI_API_KEY").unwrap();

    let client = reqwest::Client::new();

    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&serde_json::json!({
            "model": "gpt-4o-mini",
            "messages": [
                {"role": "user", "content": prompt}
            ]
        }))
        .send()
        .await
        .unwrap();

    let json: serde_json::Value = res.json().await.unwrap();

    json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn play(Json(input): Json<GameInput>) -> Json<GameOutput> {
    let reply = call_llm(&input.message).await;
    Json(GameOutput { reply })
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::permissive();

    let app = Router::new()
        .route("/play", post(play))
        .layer(cors);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap();

    let addr = format!("0.0.0.0:{}", port);

    println!("Running on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();
}
