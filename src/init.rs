use dotenv;
use std::{env, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    config::Config,
    game::GameState,
    state::{self, AppState},
};

pub fn init() -> AppState {
    dotenv::dotenv().ok();
    let groq_api_key = env::var("GROQ_API_KEY").expect("Groq API key not set");
    return AppState {
        state: Arc::new(Mutex::new(state::State {
            config: Config { groq_api_key },
            game_state: GameState::init(),
        })),
    };
}
