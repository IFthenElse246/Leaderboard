use indexset::BTreeSet;
use rocket::{tokio, Ignite, Orbit, Rocket};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Seek, Write};
use std::sync::Mutex;
use std::time::Duration;

use crate::util;

#[derive(Clone)]
pub struct User {
    pub board: String,
    pub write: bool,
}

pub struct AppState {
    pub boards: Mutex<HashMap<String, BTreeSet<Entry<f64>>>>,
    pub api_keys: Mutex<HashMap<String, User>>,
    pub port: usize,
    pub save_interval: u64
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
    pub leaderboards: HashMap<String, ConfigBoard>,
    pub save_interval: u64
}

impl AppState {
    pub fn new(mut file: &std::fs::File) -> Self {
        let content;

        if file.metadata().unwrap().len() == 0 {
            content = include_str!("default_config.json").to_string();
            let _ = file.write_all(content.as_bytes());
        } else {
            content = util::read_file(&file).expect("Failed to read config file content");
        }

        file.rewind().expect("Failed to read config file content");

        let json =
            serde_json::from_str::<Config>(content.as_str()).expect("Invalid config file, delete to return to default config.");

        let mut boards = HashMap::new();
        let mut keys = HashMap::new();

        for (name, board) in json.leaderboards {
            let set: BTreeSet<Entry<f64>> = BTreeSet::new();
            boards.insert(name.clone(), set);
            for (key, user) in board.keys {
                keys.insert(key, User { board: name.clone(), write: user.write });
            }
        }

        Self {
            boards: Mutex::new(boards),
            api_keys: Mutex::new(keys),
            port: json.port,
            save_interval: json.save_interval
        }
    }
}

#[derive(PartialEq)]
pub struct Entry<T: PartialOrd> {
    user_id: u64,
    points: T,
    timestamp: usize,
}

impl<T: PartialOrd> PartialOrd for Entry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.points != other.points {
            return self.points.partial_cmp(&other.points);
        }
        return other.timestamp.partial_cmp(&self.timestamp);
    }
}

impl<T: PartialOrd> Ord for Entry<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.points.partial_cmp(&other.points) {
            None => std::cmp::Ordering::Equal,
            Some(std::cmp::Ordering::Equal) => other.timestamp.cmp(&self.timestamp),
            Some(x) => x,
        }
    }
}

impl<T: PartialOrd> Eq for Entry<T> {}

pub async fn save_loop() {
    //let state = r.state::<AppState>().expect("Failed to get state for save loop.");
    loop {
        let stdout = io::stdout();
        let _ = writeln!(&mut stdout.lock(), "test");
        tokio::time::sleep(Duration::from_secs(2)).await;
        
    }
}