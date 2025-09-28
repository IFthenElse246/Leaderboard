use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Seek, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::backend::User;
use crate::board::Board;
use crate::util;

#[derive(Serialize, Deserialize)]
pub struct ConfigBoard {
    pub keys: HashMap<String, ConfigUser>,
    pub cap: Option<usize>
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

pub struct AppState {
    pub boards: Mutex<HashMap<String, Board<i64, f64>>>,
    pub api_keys: Mutex<HashMap<String, User>>,
    pub port: usize,
    pub save_interval: u64,
    pub boards_file: Mutex<File>,
    pub saves_path: PathBuf
}

impl AppState {
    pub fn new(
        mut file: &std::fs::File,
        mut boards_file: std::fs::File,
        saves_path: &PathBuf,
    ) -> Self {
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
            let save_path = saves_path.join(format!("{name}.board"));
            let alt_path = saves_path.join(format!("{name}_saving.part"));

            let mut board;

            if !save_path.exists() && alt_path.exists() {
                let result = std::fs::rename(&alt_path, &save_path);
                if let Err(err) = result {
                    panic!("Failed to recover save file.\n{}", err)
                }
            }

            if save_path.exists() {
                let save_file = match File::open(save_path.clone()) {
                    Err(err) => {
                        panic!(
                            "Failed to read file ({}) for leaderboard {name}\n{err}",
                            match save_path.to_str() {
                                Some(path) => path.to_string(),
                                None => format!("/saves/{name}.board"),
                            }
                        );
                    }
                    Ok(v) => v,
                };
                let mut buf_reader = BufReader::new(save_file);

                let tree =
                    match bincode::decode_from_std_read(&mut buf_reader, bincode::config::standard()) {
                        Err(err) => {
                            panic!(
                                "Failed to parse file ({}) for leaderboard {name}\n{err}",
                                match save_path.to_str() {
                                    Some(path) => path.to_string(),
                                    None => format!("/saves/{name}.board"),
                                }
                            );
                        }
                        Ok(tree) => tree,
                    };

                board = Board::from_tree(tree);

                if alt_path.exists() {
                    let _ = std::fs::remove_file(alt_path);
                }
            } else {
                board = Board::new();
                let _ = writeln!(&mut io::stdout().lock(), "Failed to find .board file for board {name}. Using empty board.");
            }

            if let Some(cap) = json_board.cap {
                board.set_size_cap(cap);
            } else {
                board.remove_size_cap();
            }
            
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

        Self {
            boards: Mutex::new(boards),
            api_keys: Mutex::new(keys),
            port: json.port,
            save_interval: json.save_interval,
            boards_file: Mutex::new(boards_file),
            saves_path: saves_path.clone()
        }
    }

    fn write_boards_json(&self) {
        let mut json: HashMap<String, ConfigBoard> = HashMap::new();
        let users = self.api_keys.lock().unwrap();

        for (k, user) in users.iter() {
            let board_name = user.board.clone();
            let binding = self.boards.lock().unwrap();
            let actual_board = binding.get(&board_name);
            let cap = match actual_board {
                None => None,
                Some(v) => v.get_size_cap()
            };
            if !json.contains_key(&board_name) {
                let board = ConfigBoard {
                    keys: HashMap::new(),
                    cap: cap
                };
                json.insert(board_name.clone(), board);
            }
            let json_board = json.get_mut(&board_name).unwrap();
            json_board.cap = cap;
            json_board
                .keys
                .insert(k.to_string(), ConfigUser { write: user.write });
        }

        let boards = self.boards.lock().unwrap();
        for (board_name, board) in boards.iter() {
            if !json.contains_key(board_name) {
                let board = ConfigBoard {
                    keys: HashMap::new(),
                    cap: board.get_size_cap()
                };
                json.insert(board_name.clone(), board);
            }
        }

        let mut file = self
            .boards_file
            .lock()
            .expect("Could not update the boards file.");
        let _ = file.set_len(0);
        file.write_all(serde_json::to_string_pretty(&json).unwrap().as_bytes())
            .expect("Could not update the boards file.");
        file.rewind().expect("Could not update the boards file.");
    }

    pub fn create_board(&self, name: String) -> bool {
        if self.boards.lock().unwrap().contains_key(&name) {
            return false;
        }

        let save_path = self.saves_path.join(format!("{name}.board"));
        let board;

        if save_path.exists() {
            let save_file = match File::open(save_path.clone()) {
                Err(err) => {
                    panic!(
                        "Failed to read file ({}) for leaderboard {name}\n{err}",
                        match save_path.to_str() {
                            Some(path) => path.to_string(),
                            None => format!("/saves/{name}.board"),
                        }
                    );
                }
                Ok(v) => v,
            };
            let mut buf_reader = BufReader::new(save_file);

            let tree =
                match bincode::decode_from_std_read(&mut buf_reader, bincode::config::standard()) {
                    Err(err) => {
                        panic!(
                            "Failed to parse file ({}) for leaderboard {name}\n{err}",
                            match save_path.to_str() {
                                Some(path) => path.to_string(),
                                None => format!("/saves/{name}.board"),
                            }
                        );
                    }
                    Ok(tree) => tree,
                };
            board = Board::from_tree(tree);
        } else {
            board = Board::new()
        }

        self.boards.lock().unwrap().insert(name, board);
        self.write_boards_json();
        return true;
    }

    pub fn set_board_cap(&self, board: &String, cap: usize) -> bool {
        let mut boards = self.boards.lock().unwrap();

        let board = match boards.get_mut(board) {
            Some(v) => v,
            None => {return false;}
        };
        board.set_size_cap(cap);
        let _ = drop(boards);
        self.write_boards_json();
        return true;
    }

    pub fn rem_board_cap(&self, board: &String) -> bool {
        let mut boards = self.boards.lock().unwrap();

        let board = match boards.get_mut(board) {
            Some(v) => v,
            None => {return false;}
        };
        board.remove_size_cap();
        let _ = drop(boards);
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
        let _ = drop(boards);
        let _ = drop(users);

        let save_path = self.saves_path.join(format!("{name}.board"));
        if save_path.exists() {
            let _ = std::fs::remove_file(save_path);
        }
        
        self.write_boards_json();
        return true;
    }

    pub fn create_key(&self, api_key: String, board: String, write: bool) -> bool {
        let mut users = self.api_keys.lock().unwrap();
        if users.contains_key(&api_key) {
            return false;
        }
        users.insert(
            api_key,
            User {
                board: board,
                write: write,
            },
        );
        let _ = drop(users);
        self.write_boards_json();
        return true;
    }

    pub fn delete_key(&self, api_key: &String) -> bool {
        let mut users = self.api_keys.lock().unwrap();
        if !users.contains_key(api_key) {
            return false;
        }
        users.remove(api_key);
        let _ = drop(users);
        self.write_boards_json();
        return true;
    }

    pub fn set_key_write_perms(&self, api_key: &String, write: bool) -> bool {
        let mut users = self.api_keys.lock().unwrap();
        if !users.contains_key(api_key) {
            return false;
        }
        users.get_mut(api_key).unwrap().write = write;
        let _ = drop(users);
        self.write_boards_json();
        return true;
    }
}
