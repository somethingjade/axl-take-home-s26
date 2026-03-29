use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{config::Config, game::GameState};

#[derive(Clone, Debug)]
pub struct AppState {
    pub state: Arc<Mutex<State>>,
}

#[derive(Clone, Debug)]
pub struct State {
    pub config: Config,
    pub game_state: GameState,
}
