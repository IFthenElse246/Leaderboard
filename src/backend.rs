use rocket::http::Status;
use rocket::tokio;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::hash::Hash;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::app_state::AppState;

#[derive(Clone)]
pub struct User {
    pub board: String,
    pub write: bool,
}

pub struct Interaction<'r> {
    pub user: User,
    pub state: &'r rocket::State<Arc<AppState>>,
}

pub fn save(state_arc: &Arc<AppState>, saves_path: &PathBuf) {
    let stdout = io::stdout();
    let _ = writeln!(&mut stdout.lock(), "Starting backup");

    let temp_path = saves_path.join("temp");

    let queue_lock = state_arc.boards.lock().unwrap();

    let mut queue = Vec::with_capacity(queue_lock.len());
    for (name, _board) in queue_lock.iter() {
        queue.push(name.clone());
    }
    drop(queue_lock);

    for name in queue.iter() {
        let lock = state_arc.boards.lock().unwrap();

        if let Some(board) = lock.get(name) {
            let _ = writeln!(&mut stdout.lock(), "Saving {name}...");
            let new_tree = board.get_tree_copy();

            drop(lock);

            let handle = match File::create(&temp_path) {
                Ok(v) => v,
                Err(err) => {
                    let _ = writeln!(
                        &mut io::stderr().lock(),
                        "Failed to open temp file to save leaderboard backup.\n{}",
                        err
                    );
                    break;
                }
            };

            match ciborium::into_writer(&new_tree, handle) {
                Ok(_) => {
                    if let Err(err) =
                        std::fs::rename(&temp_path, saves_path.join(format!("{name}.cbor")))
                    {
                        let _ = writeln!(
                            &mut io::stderr().lock(),
                            "Failed to rename temp file into save.\n{}",
                            err
                        );
                        break;
                    }
                }
                Err(err) => {
                    let _ = writeln!(
                        &mut io::stderr().lock(),
                        "Failed to write to temp file to save leaderboard backup.\n{}",
                        err
                    );
                    break;
                }
            }
        }
    }

    let _ = writeln!(&mut stdout.lock(), "All boards saved.");
}

pub async fn save_loop(state_arc: Arc<AppState>, saves_path: &PathBuf) {
    let interval = state_arc.save_interval;

    loop {
        tokio::time::sleep(Duration::from_secs(interval)).await;
        save(&state_arc, saves_path);
        crate::cli::put_cli_prompt();
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ActionType {
    Update,
}

pub fn execute_action(
    action: ActionType,
    interaction: &Interaction,
    dat: String,
) -> Result<String, Status> {
    match action {
        ActionType::Update => execute_update(interaction, dat),
    }
}

#[derive(Serialize, Deserialize)]
struct UpdReq {
    id: u64,
    value: f64,
}

pub fn execute_update(interaction: &Interaction, dat: String) -> Result<String, Status> {
    let json_res = serde_json::from_str::<UpdReq>(dat.as_str());
    if json_res.is_err() {
        return Err(Status::BadRequest);
    }
    let json = json_res.unwrap();
    match update_entry(interaction, json.id, json.value) {
        true => Ok(format!("Successfully updated {0}.", json.id)),
        false => Ok(format!("Added player {0} and updated.", json.id)),
    }
}

pub fn update_entry(interaction: &Interaction, id: u64, value: f64) -> bool {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.update_entry(id, value)
}

pub fn get_points(interaction: &Interaction, id: u64) -> Option<f64> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    Some(board.get_entry(id)?.points)
}

pub fn get_size(interaction: &Interaction) -> usize {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_size()
}

pub fn get_rank(interaction: &Interaction, id: u64) -> Option<usize> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_rank(id)
}

pub fn clear(interaction: &Interaction) {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.clear()
}
