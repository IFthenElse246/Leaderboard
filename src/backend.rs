use indexset::BTreeSet;
use rocket::http::Status;
use rocket::tokio;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{self, Seek, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn current_time() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub struct Board<T: PartialOrd = f64> {
    tree: BTreeSet<Arc<Entry<T>>>,
    map: HashMap<u64, Arc<Entry<T>>>,
}

impl<T: PartialOrd> Board<T> {
    pub fn get_entry(&self, id: u64) -> Option<&Arc<Entry<T>>> {
        return self.map.get(&id);
    }

    pub fn add_entry(&mut self, entry: Entry<T>) -> bool {
        let id = entry.user_id;
        let arc = Arc::new(entry);
        let arc2 = arc.clone();

        if self.map.contains_key(&id) {
            return false;
        }

        self.tree.insert(arc);
        self.map.insert(id, arc2);
        return true;
    }

    pub fn remove_entry(&mut self, id: u64) -> Option<Arc<Entry<T>>> {
        let option = self.map.remove(&id);
        if let None = option {
            return None;
        }
        let entry = option.unwrap();
        self.tree.remove(&entry);
        Some(entry)
    }

    fn remove_entry_(&mut self, entry: Arc<Entry<T>>) -> bool {
        return self.tree.remove(&entry);
    }

    pub fn update_entry(&mut self, id: u64, points: T) -> bool {
        let old_entry_opt = self.get_entry(id);
        if let None = old_entry_opt {
            self.add_entry(Entry {
                user_id: id,
                points: points,
                timestamp: current_time(),
            });
            return false;
        }
        let old_entry = old_entry_opt.unwrap();
        let mut timestamp = old_entry.timestamp;
        if old_entry.points == points {
            return true;
        } else if old_entry.points < points {
            timestamp = current_time();
        }
        self.remove_entry_(old_entry.clone());
        self.add_entry(Entry {
            user_id: id,
            points: points,
            timestamp: timestamp,
        });
        true
    }

    pub fn get_rank(&self, id: u64) -> Option<usize> {
        let entry = self.get_entry(id)?;
        return Some(self.tree.rank(entry));
    }

    pub fn get_size(&self) -> usize {
        self.tree.len()
    }

    pub fn get_top(&self, count: usize) -> Vec<Arc<Entry<T>>> {
        let mut ret = Vec::new();

        let mut iter = self.tree.iter();
        for _i in 1..=count {
            match iter.next() {
                Some(entry) => {
                    ret.push(entry.clone());
                }
                None => {
                    break;
                }
            }
        }

        return ret;
    }

    pub fn get_bottom(&self, count: usize) -> Vec<Arc<Entry<T>>> {
        let mut ret = Vec::new();

        let mut iter = self.tree.iter().rev();
        for _i in 1..=count {
            match iter.next() {
                Some(entry) => {
                    ret.push(entry.clone());
                }
                None => {
                    break;
                }
            }
        }

        return ret;
    }

    fn new() -> Self {
        Self {
            tree: BTreeSet::new(),
            map: HashMap::new(),
        }
    }

    pub fn get_after(&self, id: u64, count: usize) -> Result<Vec<Arc<Entry<T>>>, String> {
        let entry = self
            .get_entry(id)
            .ok_or_else(|| format!("Id '{0}' not in leaderboard.", id))?;
        let mut ret = Vec::new();

        let mut first = true;
        for v in self
            .tree
            .range::<std::ops::RangeFrom<Arc<Entry<T>>>, Arc<Entry<T>>>(entry.clone()..)
        {
            if first {
                first = false;
                continue;
            }
            if ret.len() >= count {
                break;
            }
            ret.push(v.clone());
        }

        Ok(ret)
    }
}

use crate::util;

#[derive(Clone)]
pub struct User {
    pub board: String,
    pub write: bool,
}

pub struct Interaction<'r> {
    pub user: User,
    pub state: &'r rocket::State<Arc<AppState>>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ActionType {
    Update,
}

pub struct AppState {
    pub boards: Mutex<HashMap<String, Board<f64>>>,
    pub api_keys: Mutex<HashMap<String, User>>,
    pub port: usize,
    pub save_interval: u64,
    pub boards_file: Mutex<File>,
    pub actions: HashMap<ActionType, fn(Interaction, String) -> Result<String, Status>>,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigBoard {
    pub keys: HashMap<String, ConfigUser>,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigUser {
    pub write: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub port: usize,
    pub save_interval: u64,
}

#[derive(Serialize, Deserialize)]
struct UpdReq {
    id: u64,
    value: f64,
}

impl AppState {
    pub fn new(mut file: &std::fs::File, mut boards_file: std::fs::File) -> Self {
        let content;
        let board_content;

        if file.metadata().unwrap().len() == 0 {
            content = include_str!("default_config.json").to_string();
            let _ = file.write_all(content.as_bytes());
        } else {
            content = util::read_file(&file).expect("Failed to read config file content");
        }

        if boards_file.metadata().unwrap().len() == 0 {
            board_content = include_str!("default_boards.json").to_string();
            let _ = boards_file.write_all(board_content.as_bytes());
        } else {
            board_content =
                util::read_file(&boards_file).expect("Failed to read board file content");
        }

        file.rewind().expect("Failed to read config file content");
        boards_file
            .rewind()
            .expect("Failed to read board file content");

        let json = serde_json::from_str::<Config>(content.as_str())
            .expect("Invalid config file, delete to return to default config.");
        let board_json =
            serde_json::from_str::<HashMap<String, ConfigBoard>>(&board_content.as_str())
                .expect("Invalid boards file, delete to return to default config.");

        let mut boards = HashMap::new();
        let mut keys = HashMap::new();

        for (name, json_board) in board_json {
            let board = Board::new();
            boards.insert(name.clone(), board);
            for (key, user) in json_board.keys {
                keys.insert(
                    key,
                    User {
                        board: name.clone(),
                        write: user.write,
                    },
                );
            }
        }

        let mut actions_map: HashMap<
            ActionType,
            fn(Interaction, String) -> Result<String, Status>,
        > = HashMap::new();
        actions_map.insert(
            ActionType::Update,
            |interaction, dat| -> Result<String, Status> {
                let json_res = serde_json::from_str::<UpdReq>(dat.as_str());
                if json_res.is_err() {
                    return Err(Status::BadRequest);
                }
                let json = json_res.unwrap();
                match update_entry(interaction, json.id, json.value) {
                    true => Ok(format!("Successfully updated {0}.", json.id)),
                    false => Ok(format!("Added player {0} and updated.", json.id)),
                }
            },
        );

        Self {
            boards: Mutex::new(boards),
            api_keys: Mutex::new(keys),
            port: json.port,
            save_interval: json.save_interval,
            boards_file: Mutex::new(boards_file),
            actions: actions_map,
        }
    }

    fn write_boards_json(&self) {
        let mut json: HashMap<String, ConfigBoard> = HashMap::new();
        let users = self.api_keys.lock().unwrap();

        for (k, user) in users.iter() {
            let board_name = user.board.clone();
            if !json.contains_key(&board_name) {
                let board = ConfigBoard {
                    keys: HashMap::new(),
                };
                json.insert(board_name.clone(), board);
            }
            let json_board = json.get_mut(&board_name).unwrap();
            json_board
                .keys
                .insert(k.to_string(), ConfigUser { write: user.write });
        }

        let mut file = self
            .boards_file
            .lock()
            .expect("Could not update the boards file.");
        file.write_all(serde_json::to_string_pretty(&json).unwrap().as_bytes())
            .expect("Could not update the boards file.");
        file.rewind().expect("Could not update the boards file.");
    }

    pub fn create_board(&self, name: &String) -> bool {
        let mut boards = self.boards.lock().unwrap();
        if boards.contains_key(name) {
            return false;
        }
        boards.insert(name.clone(), Board::new());
        self.write_boards_json();
        return true;
    }

    pub fn delete_board(&self, name: &String) -> bool {
        let mut boards = self.boards.lock().unwrap();
        if !boards.contains_key(name) {
            return false;
        }
        boards.remove(name);

        let mut users = self.api_keys.lock().unwrap();
        users.retain(|_k, usr| -> bool { usr.board != *name });

        self.write_boards_json();
        return true;
    }

    pub fn create_user(&self, api_key: &String, board: &String, write: bool) -> bool {
        let mut users = self.api_keys.lock().unwrap();
        if users.contains_key(api_key) {
            return false;
        }
        users.insert(
            api_key.clone(),
            User {
                board: board.clone(),
                write: write,
            },
        );
        self.write_boards_json();
        return true;
    }

    pub fn delete_user(&self, api_key: &String) -> bool {
        let mut users = self.api_keys.lock().unwrap();
        if !users.contains_key(api_key) {
            return false;
        }
        users.remove(api_key);
        self.write_boards_json();
        return true;
    }

    pub fn set_user_write_perms(&self, api_key: &String, write: bool) -> bool {
        let mut users = self.api_keys.lock().unwrap();
        if !users.contains_key(api_key) {
            return false;
        }
        users.get_mut(api_key).unwrap().write = write;
        self.write_boards_json();
        return true;
    }
}

#[derive(PartialEq)]
pub struct Entry<T>
where
    T: PartialOrd + ?Sized,
{
    user_id: u64,
    timestamp: u128,
    points: T,
}

impl<T: PartialOrd> PartialOrd for Entry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.points != other.points {
            return self.points.partial_cmp(&other.points);
        }
        if other.timestamp != self.timestamp {
            return other.timestamp.partial_cmp(&self.timestamp);
        }
        return other.user_id.partial_cmp(&self.user_id);
    }
}

impl<T: PartialOrd> Ord for Entry<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.points.partial_cmp(&other.points) {
            None | Some(std::cmp::Ordering::Equal) => match other.timestamp.cmp(&self.timestamp) {
                std::cmp::Ordering::Equal => other.user_id.cmp(&self.user_id),
                x => x,
            },
            Some(x) => x,
        }
    }
}

impl<T: PartialOrd> Eq for Entry<T> {}

pub async fn save_loop(state_arc: Arc<AppState>) {
    let interval = state_arc.save_interval;

    loop {
        tokio::time::sleep(Duration::from_secs(interval)).await;
        let stdout = io::stdout();
        let _ = writeln!(&mut stdout.lock(), "Saving backup...");
    }
}

pub fn update_entry(interaction: Interaction, id: u64, value: f64) -> bool {
    let mut binding = interaction.state.boards.lock().unwrap();
    let board = binding.get_mut(&interaction.user.board).unwrap();
    board.update_entry(id, value)
}
