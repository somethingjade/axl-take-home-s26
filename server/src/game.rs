use rand::RngExt;
use uuid::Uuid;
use std::{
    collections::{HashMap, HashSet},
    fs,
};

use serde::{Deserialize, Serialize};

use crate::{
    consts::{BOT_INTRO_BASE, INTRO_BASE},
    llm::{build_prompt, call_llm},
    state,
};

#[derive(Clone, Debug)]
pub enum Round {
    Start,
    Introductions,
    Question(u32),
    Answer(u32),
    Discussion,
    Voting,
    End,
}

#[derive(Deserialize, Serialize)]
struct BotPrompt {
    personality: String,
    playstyle: String,
    character_summary: String,
    behavioural_directive: String,
}

#[derive(Deserialize)]
struct PromptPair {
    prompt: String,
    fake_prompt: String,
}

#[derive(Clone, Debug)]
pub struct Bot {
    system_prompt: String,
}

#[derive(Clone, Debug)]
pub struct GameState {
    player_count: usize,
    eliminated_players: HashSet<u32>,
    impostor: u32,
    prompt: String,
    fake_prompt: String,
    round: Round,
    player_prompt: String,
    log: String,
    bots: Vec<Bot>,
}

impl GameState {
    pub fn init() -> Self {
        let mut rng = rand::rng();
        let player_count = 4;
        let eliminated_players = HashSet::new();
        let impostor = (rng.random::<u32>() as usize % player_count) as u32 + 1;
        let data = fs::read_to_string("assets/prompts.json").expect("[ERROR] Prompt dataset not found");
        let parsed: Vec<PromptPair> = serde_json::from_str(&data).unwrap();
        let prompt_idx = rng.random::<u32>() as usize % parsed.len();
        let prompt_pair: &PromptPair = &parsed[prompt_idx];
        let prompt = prompt_pair.prompt.to_owned();
        let fake_prompt = prompt_pair.fake_prompt.to_owned();
        let mut ret = Self {
            player_count,
            eliminated_players,
            impostor,
            prompt,
            fake_prompt,
            round: Round::Start,
            player_prompt: String::new(),
            log: String::new(),
            bots: Vec::new(),
        };
        ret.load_bots();
        return ret;
    }

    pub fn done(&self) -> bool {
        matches!(self.round, Round::End)
    }

    fn load_bots(&mut self) {
        let data = fs::read_to_string("assets/bot_dataset.json").expect("[ERROR] Bot dataset not found");
        let parsed: Vec<BotPrompt> = serde_json::from_str(&data).unwrap();
        let mut used = HashSet::<usize>::new();
        let mut rng = rand::rng();
        for i in 0..self.player_count - 1 {
            let id = i as u32 + 2;
            let mut idx = rng.random::<u32>() as usize % parsed.len();
            while used.contains(&idx) {
                idx = rng.random::<u32>() as usize % parsed.len();
            }
            used.insert(idx);
            let bot_prompt = serde_json::to_string(&parsed[idx]).unwrap();
            let system_prompt = format!(
                "You are a player in a social deduction game. Here are instructions for the game: {}. When answering questions, always act as if you have an experience related to the prompt. Strictly follow this behavioural directive in all responses: {}.",
                self.bot_intro(id),
                bot_prompt
            );
            self.bots.push(Bot { system_prompt });
        }
    }

    pub fn intro(&self, id: u32) -> String {
        let prompt_string = if id == self.impostor {
            format!(
                "You are the impostor.\nYour prompt is: {}.\nDon't get caught!",
                self.fake_prompt
            )
        } else {
            format!(
                "You are not the impostor.\nThe prompt is: {}.",
                self.prompt
            )
        };
        return format!("{INTRO_BASE}\n\nYou are player #{id}\n{prompt_string}");
    }

    pub fn bot_intro(&self, id: u32) -> String {
        let prompt_string = if id == self.impostor {
            format!(
                "You are the impostor.\nYour prompt is: {}.\nDon't get caught!",
                self.fake_prompt
            )
        } else {
            format!(
                "You are not the impostor.\nThe prompt is: {}.\n",
                self.prompt
            )
        };
        return format!("{BOT_INTRO_BASE}\n\nYou are player #{id}\n{prompt_string}");
    }
}

// This function is a mess.
pub async fn run(state: &mut state::State, session_id: &Uuid, input: &String) -> String {
    let config = &state.config;
    let game_state = match state.sessions.get_mut(session_id) {
        Some(val) => val,
        None => {
            return "".to_string();
        },
    };
    let mut ret = String::new();
    loop {
        match game_state.round {
            Round::Start => {
                ret = game_state.intro(1);
                let prompt =
                    "Game Master: First, players, please introduce yourselves!".to_string();
                game_state.round = Round::Introductions;
                game_state.player_prompt = prompt.to_owned();
                ret.push_str(format!("\n\n{prompt}").as_str());
                return ret;
            }
            Round::Introductions => {
                ret = format!("\n\nPlayer 1: {input}");
                for i in 0..game_state.player_count - 1 {
                    let id = i + 2;
                    let prompt = build_prompt(&game_state.log, &game_state.player_prompt);
                    let bot_response =
                        call_llm(config, &game_state.bots[i].system_prompt, &prompt).await;
                    let bot_text = bot_response.text;
                    ret.push_str(format!("\n\nPlayer {id}: {bot_text}").as_str());
                }
                game_state.log.push_str(&game_state.player_prompt);
                game_state.log.push_str(&ret);
                game_state.round = Round::Question(1);
                game_state.player_prompt =
                    "\n\nGame Master: Player 1, your turn to ask a question!".to_owned();
                ret.push_str(&game_state.player_prompt);
                return ret;
            }
            Round::Question(id) => {
                let old_ret = ret;
                ret = String::new();
                if id == 1 {
                    ret.push_str(format!("\n\nPlayer 1: {input}").as_str());
                } else {
                    let prompt = build_prompt(&game_state.log, &game_state.player_prompt);
                    let bot_response = call_llm(
                        config,
                        &game_state.bots[id as usize - 2].system_prompt,
                        &prompt,
                    )
                    .await;
                    let bot_text = bot_response.text;
                    ret.push_str(format!("\n\nPlayer {}: {}", id, bot_text).as_str());
                };
                game_state.log.push_str(&game_state.player_prompt);
                game_state.log.push_str(format!("{ret}").as_str());
                ret = old_ret + &ret;
                game_state.round = Round::Answer(id);
                game_state.player_prompt =
                    "\n\nGame Master: Other players, your turn to answer!".to_string();
                ret.push_str(&game_state.player_prompt);
                if id != 1 {
                    return ret;
                }
            }
            Round::Answer(mut id) => {
                let old_ret = ret;
                ret = String::new();
                if id != 1 {
                    ret.push_str(format!("\n\nPlayer 1: {input}").as_str());
                }
                for i in 1..game_state.player_count {
                    let bot_id = i as u32 + 1;
                    if game_state.eliminated_players.contains(&bot_id) || bot_id == id {
                        continue;
                    }
                    let prompt = build_prompt(&game_state.log, &game_state.player_prompt);
                    let bot_response = call_llm(
                        config,
                        &game_state.bots[bot_id as usize - 2].system_prompt,
                        &prompt,
                    )
                    .await;
                    let bot_text = bot_response.text;
                    ret.push_str(format!("\n\nPlayer {}: {}", bot_id, bot_text).as_str());
                }
                game_state.log.push_str(&game_state.player_prompt);
                game_state.log.push_str(format!("{ret}").as_str());
                ret = old_ret + &ret;
                loop {
                    if id == game_state.player_count as u32 {
                        game_state.round = Round::Discussion;
                        game_state.player_prompt ="\n\nGame Master: It's almost time to vote! Players, please provide your thoughts before we proceed to voting.".to_string();
                        ret.push_str(&game_state.player_prompt);
                        return ret;
                    } else {
                        id += 1;
                        if !game_state.eliminated_players.contains(&id) {
                            game_state.round = Round::Question(id);
                            game_state.player_prompt = format!(
                                "\n\nGame Master: Player {id}, your turn to ask a question!"
                            );
                            ret.push_str(&game_state.player_prompt);
                            break;
                        }
                    }
                }
            }
            Round::Discussion => {
                ret.push_str(format!("\n\nPlayer 1: {input}").as_str());
                for i in 1..game_state.player_count {
                    let bot_id = i as u32 + 1;
                    if game_state.eliminated_players.contains(&bot_id) {
                        continue;
                    }
                    let prompt = build_prompt(&game_state.log, &game_state.player_prompt);
                    let bot_response = call_llm(
                        config,
                        &game_state.bots[bot_id as usize - 2].system_prompt,
                        &prompt,
                    )
                    .await;
                    let bot_text = bot_response.text;
                    ret.push_str(format!("\n\nPlayer {}: {}", bot_id, bot_text).as_str());
                }
                game_state.log.push_str(&game_state.player_prompt);
                game_state.log.push_str(format!("{ret}").as_str());
                game_state.round = Round::Voting;
                game_state.player_prompt = "\n\nGame Master: It's time to vote! Please state the number of the player who you believe to be the impostor.".to_string();
                ret.push_str(&game_state.player_prompt);
                return ret;
            }
            Round::Voting => {
                let player_vote = match input.parse::<u32>() {
                    Ok(val)
                        if val > 1
                            && val <= game_state.player_count as u32
                            && !game_state.eliminated_players.contains(&val) =>
                    {
                        val
                    }
                    _ => {
                        return "\n\nPlease enter a valid vote!".to_string();
                    }
                };
                let mut votes = HashMap::from([(player_vote, 1)]);
                ret.push_str(
                    format!("\n\nGame Master: Player 1 voted: Player {player_vote}").as_str(),
                );
                for i in 1..game_state.player_count {
                    let id = i as u32 + 1;
                    if game_state.eliminated_players.contains(&id) {
                        continue;
                    }
                    let prompt = build_prompt(&game_state.log, &game_state.player_prompt);
                    let bot_response = call_llm(
                        config,
                        &game_state.bots[id as usize - 2].system_prompt,
                        &prompt,
                    )
                    .await;
                    let bot_vote = bot_response
                        .vote
                        .expect("[ERROR] Bot did not vote during voting round");
                    match votes.get_mut(&bot_vote) {
                        Some(val) => *val += 1,
                        None => {
                            votes.insert(bot_vote, 1);
                        }
                    }
                    ret.push_str(
                        format!("\n\nGame Master: Player {} voted: Player {}", id, bot_vote)
                            .as_str(),
                    );
                }
                let mut max = 0;
                let mut max_voted = Vec::new();
                for (player, votes) in votes {
                    if votes == max {
                        max_voted.push(player);
                    } else if votes > max {
                        max = votes;
                        max_voted = vec![player];
                    }
                }
                let eliminate_idx = (rand::random::<u32>() as usize) % max_voted.len();
                let eliminate = max_voted[eliminate_idx];
                game_state.eliminated_players.insert(eliminate);
                ret.push_str(
                    format!("\n\nGame Master: Player {eliminate} has been voted out!").as_str(),
                );
                let impostor_eliminated = if eliminate == game_state.impostor {
                    ret.push_str(format!("\n\nGame Master: They were the impostor!").as_str());
                    true
                } else {
                    ret.push_str(format!("\n\nGame Master: They were not the impostor!").as_str());
                    false
                };
                if eliminate == 1 {
                    ret.push_str(format!("\n\nYou were voted out - game over!").as_str());
                    if game_state.impostor == 1 {
                        ret.push_str(format!("\nThe prompt was: {}", game_state.prompt).as_str());
                    } else {
                        ret.push_str(format!("\nPlayer {} was the impostor", game_state.impostor).as_str());
                        ret.push_str(format!("\nThe impostor's prompt was: {}", game_state.fake_prompt).as_str());

                    }
                    game_state.round = Round::End;
                    return ret;
                }
                if impostor_eliminated {
                    ret.push_str("\n\nGame Master: Congratulations! You have successfully voted out the impostor!");
                    ret.push_str(format!("\nThe impostor's prompt was: {}", game_state.fake_prompt).as_str());
                    game_state.round = Round::End;
                    return ret;
                }
                if !impostor_eliminated
                    && game_state.eliminated_players.len() == game_state.player_count - 2
                {
                    ret.push_str(format!("\n\nGame Master: There are only 2 players remaining. The impostor, Player {}, wins!", game_state.impostor).as_str());
                    if game_state.impostor == 1 {
                        ret.push_str(format!("\nThe prompt was: {}", game_state.prompt).as_str());
                    } else {
                        ret.push_str(format!("\nThe impostor's prompt was: {}", game_state.fake_prompt).as_str());
                    }
                    game_state.round = Round::End;
                    return ret;
                }
                game_state.log.push_str(&game_state.player_prompt);
                game_state.log.push_str(&ret);
                game_state.round = Round::Question(1);
                game_state.player_prompt =
                    "\n\nGame Master: Player 1, your turn to ask a question!".to_owned();
                ret.push_str(&game_state.player_prompt);
                return ret;
            }
            Round::End => {
                return "".to_string();
            },
        };
    }
}
