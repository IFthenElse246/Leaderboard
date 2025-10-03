use rocket::http::Status;
use rocket::tokio;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::hash::Hash;
use std::io::{self, BufWriter, Write};
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

            let temp_path = saves_path.join(format!("{name}_saving.part"));

            let snapshot = board.get_map_snapshot();

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

            let mut buf_writer = BufWriter::new(handle);

            match bincode::encode_into_std_write(
                &snapshot,
                &mut buf_writer,
                bincode::config::standard(),
            ) {
                Ok(_) => {
                    if let Err(err) =
                        std::fs::rename(&temp_path, saves_path.join(format!("{name}.board")))
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
    id: i64,
    value: f64,
}

pub fn execute_update(interaction: &Interaction, dat: String) -> Result<String, Status> {
    let json_res = serde_json::from_str::<UpdReq>(dat.as_str());
    if json_res.is_err() {
        return Err(Status::BadRequest);
    }
    let json = json_res.unwrap();
    match update_entry(interaction, json.id, json.value) {
        Ok(b) => match b {
            true => Ok(format!("Successfully updated {0}.", json.id)),
            false => Ok(format!("Added player {0} and updated.", json.id)),
        },
        Err(v) => Ok(format!("Failed to add player {0}: {1}", json.id, v)),
    }
}

pub fn update_entry(interaction: &Interaction, id: i64, value: f64) -> Result<bool, String> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.update_entry(id, value)
}

pub fn remove_entry(interaction: &Interaction, id: i64) -> bool {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.remove_entry(&id).is_some()
}

pub fn get_points(interaction: &Interaction, id: &i64) -> Option<f64> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    Some(board.get_entry(id)?.points)
}

pub fn get_entry(interaction: &Interaction, id: &i64) -> Option<crate::board::Entry<i64, f64>> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_entry(id).map(|v| v.clone())
}

pub fn get_size(interaction: &Interaction) -> usize {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_size()
}

pub fn get_rank(interaction: &Interaction, id: &i64) -> Option<usize> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_rank(id)
}

pub fn at_rank(interaction: &Interaction, rank: usize) -> Option<crate::board::Entry<i64, f64>> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.at_rank(rank)
}

pub fn clear(interaction: &Interaction) {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.clear()
}

pub fn get_top(
    interaction: &Interaction,
    count: usize,
) -> Vec<(usize, crate::board::Entry<i64, f64>)> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_top(count)
}

pub fn get_bottom(
    interaction: &Interaction,
    count: usize,
) -> Vec<(usize, crate::board::Entry<i64, f64>)> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_bottom(count)
}

pub fn get_after(
    interaction: &Interaction,
    id: &i64,
    count: usize,
) -> Option<Vec<(usize, crate::board::Entry<i64, f64>)>> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_after(id, count)
}

pub fn get_before(
    interaction: &Interaction,
    id: &i64,
    count: usize,
) -> Option<Vec<(usize, crate::board::Entry<i64, f64>)>> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_before(id, count)
}

pub fn get_around(
    interaction: &Interaction,
    id: &i64,
    before: usize,
    after: usize,
) -> Option<Vec<(usize, crate::board::Entry<i64, f64>)>> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_around(id, before, after)
}

pub fn get_range(
    interaction: &Interaction,
    start: usize,
    end: usize,
) -> Vec<(usize, crate::board::Entry<i64, f64>)> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_range(start, end)
}
