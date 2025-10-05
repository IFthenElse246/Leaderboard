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
use crate::board::Entry;
use crate::{Key, Val};

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

    let save_locker = state_arc.save_locker.lock().unwrap();

    let stdout = io::stdout();
    let _ = writeln!(&mut stdout.lock(), "Starting backup");

    let queue_lock = state_arc.boards.lock().unwrap();

    let mut queue = Vec::with_capacity(queue_lock.len());
    for (name, _board) in queue_lock.iter() {
        queue.push(name.clone());
    }
    drop(queue_lock);

    for name in queue.iter() {
        let boards = state_arc.boards.lock().unwrap();

        if let Some(board) = boards.get(name) {
            let _ = writeln!(&mut stdout.lock(), "Saving {name}...");

            let temp_path = saves_path.join(format!("{name}_saving.part"));

            let result;

            if state_arc.lock_save {
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

                result = bincode::encode_into_std_write(
                    &board,
                    &mut buf_writer,
                    bincode::config::standard(),
                );

                let _ = drop(boards);
            } else {
                let snapshot = board.get_map_snapshot();

                let _ = drop(boards);

                let map = snapshot.get_lock().clone();

                let _ = drop(snapshot);

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

                result = bincode::encode_into_std_write(
                    map,
                    &mut buf_writer,
                    bincode::config::standard(),
                );
            }

            match result {
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

    let _ = drop(save_locker);
}

pub async fn save_loop(state_arc: Arc<AppState>, saves_path: &PathBuf) {
    let interval = state_arc.save_interval;

    loop {
        tokio::time::sleep(Duration::from_secs(interval)).await;
        save(&state_arc, saves_path);
        crate::cli::put_cli_prompt();
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionType {
    Update,
    Remove,
    Get,
    Info,
    Board,
    AtRank,
    Top,
    Bottom,
    After,
    Before,
    Around,
    Range
}

pub fn execute_action(
    action: ActionType,
    interaction: &Interaction,
    dat: String,
) -> Result<String, Status> {
    match action {
        ActionType::Update => execute_update(interaction, dat),
        ActionType::Remove => execute_remove(interaction, dat),
        ActionType::Get => execute_get(interaction, dat),
        ActionType::Info => execute_info(interaction, dat),
        ActionType::Board => execute_board(interaction, dat),
        ActionType::AtRank => execute_at_rank(interaction, dat),
        ActionType::Top => execute_top(interaction, dat),
        ActionType::Bottom => execute_bottom(interaction, dat),
        ActionType::After => execute_after(interaction, dat),
        ActionType::Before => execute_before(interaction, dat),
        ActionType::Around => execute_after(interaction, dat),
        ActionType::Range => execute_range(interaction, dat)
    }
}

#[derive(Serialize, Deserialize)]
struct UpdReq {
    id: Key,
    value: Val,
}

#[derive(Serialize, Deserialize)]
struct BasicReq {
    id: Key
}

#[derive(Serialize, Deserialize)]
struct AtRankReq {
    rank: usize
}

#[derive(Serialize, Deserialize)]
struct EdgeReq {
    count: usize,
    no_cache: Option<bool>
}

#[derive(Serialize, Deserialize)]
struct AfterBeforeReq {
    count: usize,
    id: Key
}

#[derive(Serialize, Deserialize)]
struct AroundReq {
    before: usize,
    after: usize,
    id: Key
}

#[derive(Serialize, Deserialize)]
struct RangeReq {
    start: usize,
    end: usize
}

#[derive(Serialize, Deserialize)]
struct Response {
    code: Key,
    message: String,
    entry: Option<Entry<Key, Val>>,
    rank: Option<usize>,
    entries: Option<Vec<(usize, Entry<Key, Val>)>>
}

#[derive(Serialize, Deserialize)]
pub struct BoardResponse {
    cap: Option<usize>,
    size: usize,
    min: Option<Val>
}

pub fn execute_update(interaction: &Interaction, dat: String) -> Result<String, Status> {
    if !interaction.user.write {
        return Err(Status::Forbidden)
    }

    let json_res = serde_json::from_str::<UpdReq>(dat.as_str());
    if json_res.is_err() {
        return Err(Status::BadRequest);
    }
    let json = json_res.unwrap();
    match update_entry(interaction, json.id, json.value) {
        Ok(b) => match b {
            true => Ok(serde_json::to_string(&Response {
                code: 0,
                message: format!("Successfully updated {0}.", json.id),
                entry: None,
                rank: None,
                entries: None
            }).unwrap()),
            false => Ok(serde_json::to_string(&Response {
                code: 1,
                message: format!("Added player {0} and updated.", json.id),
                entry: None,
                rank: None,
                entries: None
            }).unwrap()),
        },
        Err(v) => Ok(serde_json::to_string(&Response {
            code: 1,
            message: format!("Failed to add player {0}: {1}", json.id, v),
            entry: None,
            rank: None,
            entries: None
        }).unwrap())
    }
}

pub fn execute_remove(interaction: &Interaction, dat: String) -> Result<String, Status> {
    if !interaction.user.write {
        return Err(Status::Forbidden)
    }

    let json_res = serde_json::from_str::<BasicReq>(dat.as_str());
    if json_res.is_err() {
        return Err(Status::BadRequest);
    }
    let json = json_res.unwrap();
    match remove_entry(interaction, json.id) {
        Some(v) => Ok(serde_json::to_string(&Response {
            code: 0,
            message: format!("Successfully removed {0}.", json.id),
            entry: Some(v),
            rank: None,
            entries: None
        }).unwrap()),
        None => Ok(serde_json::to_string(&Response {
            code: 1,
            message: format!("User {0} was already not in the board.", json.id),
            entry: None,
            rank: None,
            entries: None
        }).unwrap())
    }
}

pub fn execute_get(interaction: &Interaction, dat: String) -> Result<String, Status> {
    let json_res = serde_json::from_str::<BasicReq>(dat.as_str());
    if json_res.is_err() {
        return Err(Status::BadRequest);
    }
    let json = json_res.unwrap();
    match get_entry(interaction, &json.id) {
        Some(v) => Ok(serde_json::to_string(&Response {
            code: 0,
            message: format!("Found user {0}.", json.id),
            entry: Some(v),
            rank: None,
            entries: None
        }).unwrap()),
        None => Ok(serde_json::to_string(&Response {
            code: -1,
            message: format!("User {0} was not in the board.", json.id),
            entry: None,
            rank: None,
            entries: None
        }).unwrap())
    }
}

pub fn execute_info(interaction: &Interaction, dat: String) -> Result<String, Status> {
    let json_res = serde_json::from_str::<BasicReq>(dat.as_str());
    if json_res.is_err() {
        return Err(Status::BadRequest);
    }
    let json = json_res.unwrap();
    match get_entry_and_rank(interaction, &json.id) {
        Some(v) => Ok(serde_json::to_string(&Response {
            code: 0,
            message: format!("Found user {0}.", json.id),
            entry: Some(v.1),
            rank: Some(v.0),
            entries: None
        }).unwrap()),
        None => Ok(serde_json::to_string(&Response {
            code: -1,
            message: format!("User {0} was not in the board.", json.id),
            entry: None,
            rank: None,
            entries: None
        }).unwrap())
    }
}

pub fn execute_board(interaction: &Interaction, _: String) -> Result<String, Status> {
    let res =  board_info(interaction);
    Ok(serde_json::to_string(&res).unwrap())
}

pub fn execute_at_rank(interaction: &Interaction, dat: String) -> Result<String, Status> {
    let json_res = serde_json::from_str::<AtRankReq>(dat.as_str());
    if json_res.is_err() {
        return Err(Status::BadRequest);
    }
    let json = json_res.unwrap();

    if json.rank == 0 {
        return Err(Status::BadRequest);
    }

    match at_rank(interaction, json.rank) {
        Some(v) => Ok(serde_json::to_string(&Response {
            code: 0,
            message: format!("Found user {0} with rank {1}.", v.key, json.rank),
            entry: Some(v),
            rank: Some(json.rank),
            entries: None
        }).unwrap()),
        None => Ok(serde_json::to_string(&Response {
            code: -1,
            message: format!("No user with rank {0}.", json.rank),
            entry: None,
            rank: Some(json.rank),
            entries: None
        }).unwrap())
    }
}

pub fn execute_top(interaction: &Interaction, dat: String) -> Result<String, Status> {    
    let json_res = serde_json::from_str::<EdgeReq>(dat.as_str());
    if json_res.is_err() {
        return Err(Status::BadRequest);
    }
    let json = json_res.unwrap();

    Ok(serde_json::to_string(&get_top(interaction, json.count, json.no_cache.is_some_and(|v| v))).unwrap())
}

pub fn execute_bottom(interaction: &Interaction, dat: String) -> Result<String, Status> {    
    let json_res = serde_json::from_str::<EdgeReq>(dat.as_str());
    if json_res.is_err() {
        return Err(Status::BadRequest);
    }
    let json = json_res.unwrap();

    Ok(serde_json::to_string(&get_bottom(interaction, json.count, json.no_cache.is_some_and(|v| v))).unwrap())
}

pub fn execute_after(interaction: &Interaction, dat: String) -> Result<String, Status> {    
    let json_res = serde_json::from_str::<AfterBeforeReq>(dat.as_str());
    if json_res.is_err() {
        return Err(Status::BadRequest);
    }
    let json = json_res.unwrap();

    match get_after(interaction, &json.id, json.count) {
        Some(v) => Ok(serde_json::to_string(&Response {
            code: 0,
            message: format!("Retrieved {0} entries after {1}.", v.len(), json.id),
            entry: None,
            rank: None,
            entries: Some(v)
        }).unwrap()),
        None => Ok(serde_json::to_string(&Response {
            code: -1,
            message: format!("User {0} was not in the board.", json.id),
            entry: None,
            rank: None,
            entries: None
        }).unwrap())
    }
}

pub fn execute_before(interaction: &Interaction, dat: String) -> Result<String, Status> {    
    let json_res = serde_json::from_str::<AfterBeforeReq>(dat.as_str());
    if json_res.is_err() {
        return Err(Status::BadRequest);
    }
    let json = json_res.unwrap();

    match get_before(interaction, &json.id, json.count) {
        Some(v) => Ok(serde_json::to_string(&Response {
            code: 0,
            message: format!("Retrieved {0} entries before {1}.", v.len(), json.id),
            entry: None,
            rank: None,
            entries: Some(v)
        }).unwrap()),
        None => Ok(serde_json::to_string(&Response {
            code: -1,
            message: format!("User {0} was not in the board.", json.id),
            entry: None,
            rank: None,
            entries: None
        }).unwrap())
    }
}

pub fn execute_around(interaction: &Interaction, dat: String) -> Result<String, Status> {    
    let json_res = serde_json::from_str::<AroundReq>(dat.as_str());
    if json_res.is_err() {
        return Err(Status::BadRequest);
    }
    let json = json_res.unwrap();

    match get_around(interaction, &json.id, json.before, json.after) {
        Some(v) => Ok(serde_json::to_string(&Response {
            code: 0,
            message: format!("Retrieved {0} entries around {1}.", v.len(), json.id),
            entry: None,
            rank: None,
            entries: Some(v)
        }).unwrap()),
        None => Ok(serde_json::to_string(&Response {
            code: -1,
            message: format!("User {0} was not in the board.", json.id),
            entry: None,
            rank: None,
            entries: None
        }).unwrap())
    }
}

pub fn execute_range(interaction: &Interaction, dat: String) -> Result<String, Status> {    
    let json_res = serde_json::from_str::<RangeReq>(dat.as_str());
    if json_res.is_err() {
        return Err(Status::BadRequest);
    }
    let json = json_res.unwrap();

    Ok(serde_json::to_string(&get_range(interaction, json.start, json.end)).unwrap())
}

pub fn update_entry(interaction: &Interaction, id: Key, value: Val) -> Result<bool, String> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.update_entry(id, value)
}

pub fn board_info(interaction: &Interaction) -> BoardResponse {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    BoardResponse { cap: board.get_size_cap(), size: board.get_size(), min: board.get_min() }
}

pub fn remove_entry(interaction: &Interaction, id: Key) -> Option<Entry<Key, Val>> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.remove_entry(&id)
}

pub fn get_points(interaction: &Interaction, id: &Key) -> Option<Val> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    Some(board.get_entry(id)?.points)
}

pub fn get_entry(interaction: &Interaction, id: &Key) -> Option<Entry<Key, Val>> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_entry(id).map(|v| v.clone())
}

pub fn get_entry_and_rank(interaction: &Interaction, id: &Key) -> Option<(usize, Entry<Key, Val>)> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_entry_and_rank(id)
}

pub fn get_size(interaction: &Interaction) -> usize {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_size()
}

pub fn get_rank(interaction: &Interaction, id: &Key) -> Option<usize> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_rank(id)
}

pub fn at_rank(interaction: &Interaction, rank: usize) -> Option<Entry<Key, Val>> {
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
    no_cache: bool
) -> Vec<(usize, Entry<Key, Val>)> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_top(count, no_cache, interaction.state.cache_len)
}

pub fn get_bottom(
    interaction: &Interaction,
    count: usize,
    no_cache: bool
) -> Vec<(usize, Entry<Key, Val>)> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_bottom(count, no_cache, interaction.state.cache_len)
}

pub fn get_after(
    interaction: &Interaction,
    id: &Key,
    count: usize,
) -> Option<Vec<(usize, Entry<Key, Val>)>> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_after(id, count)
}

pub fn get_before(
    interaction: &Interaction,
    id: &Key,
    count: usize,
) -> Option<Vec<(usize, Entry<Key, Val>)>> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_before(id, count)
}

pub fn get_around(
    interaction: &Interaction,
    id: &Key,
    before: usize,
    after: usize,
) -> Option<Vec<(usize, Entry<Key, Val>)>> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_around(id, before, after)
}

pub fn get_range(
    interaction: &Interaction,
    start: usize,
    end: usize,
) -> Vec<(usize, Entry<Key, Val>)> {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.get_range(start, end)
}
