use std::{
    io::{self, BufRead, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    app_state::AppState,
    backend::{self, Interaction, User},
};

fn create_interaction<'a>(
    current_user: &'a Mutex<Option<User>>,
    cmd_arc: &'a Arc<AppState>,
) -> Interaction<'a> {
    Interaction {
        user: current_user.lock().unwrap().as_ref().unwrap().clone(),
        state: (cmd_arc).into(),
    }
}

static SET_BOARD_PROMPT: &str = "No current board set, please set it with 'board <board_name>'.";

pub fn confirm_action() -> bool {
    let stdout = io::stdout();

    let _ = writeln!(
        &mut stdout.lock(),
        "Type 'y' to confirm action, anything else to cancel."
    );
    put_cli_prompt();

    let mut s = String::new();
    let res = io::stdin().lock().read_line(&mut s);

    match res {
        Ok(_) => {
            if s.trim().to_lowercase() == "y".to_string() {
                return true;
            } else {
                let _ = writeln!(&mut stdout.lock(), "Canceled.");
                return false;
            }
        }
        Err(_) => {
            let _ = writeln!(
                &mut stdout.lock(),
                "Something went wrong reading input, canceling."
            );
            return false;
        }
    };
}

pub fn execute_command(
    params: Vec<&str>,
    cmd_arc: &Arc<AppState>,
    cmd_saves_path: &PathBuf,
    current_user: &Mutex<Option<User>>,
) {
    let stdout = io::stdout();
    let cmd = params.get(0).unwrap();

    match cmd.to_lowercase().as_str() {
        "save" => {
            if params.len() > 1 {
                let _ = writeln!(&mut stdout.lock(), "Usage: save");
                return;
            }

            backend::save(&cmd_arc, cmd_saves_path);
        }
        "update" => {
            let usage_msg = "Usage: update <user_id> <points>";

            if params.len() > 3 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            let user_id = match params.get(1) {
                Some(b) => match b.parse::<i64>() {
                    Ok(id) => id,
                    Err(_) => {
                        let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                        return;
                    }
                },
                None => {
                    let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                    return;
                }
            };

            let points = match params.get(2) {
                Some(b) => match b.parse::<f64>() {
                    Ok(id) => id,
                    Err(_) => {
                        let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                        return;
                    }
                },
                None => {
                    let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                    return;
                }
            };

            if let None = *current_user.lock().unwrap() {
                let _ = writeln!(&mut stdout.lock(), "{SET_BOARD_PROMPT}");
                return;
            }

            match backend::update_entry(
                &create_interaction(&current_user, &cmd_arc),
                user_id,
                points,
            ) {
                Ok(b) => match b {
                    true => {
                        let _ = writeln!(
                            &mut stdout.lock(),
                            "Updated {user_id} to have {points} points."
                        );
                    },
                    false => {
                        let _ = writeln!(
                            &mut stdout.lock(),
                            "Added {user_id} to have {points} points."
                        );
                    }
                },
                Err(v) => {
                    let _ = writeln!(
                        &mut stdout.lock(),
                        "Failed to add {user_id}:\n{v}."
                    );
                }
            }
        }
        "remove" => {
            let usage_msg = "Usage: remove <user_id>";

            if params.len() > 2 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            let user_id = match params.get(1) {
                Some(b) => match b.parse::<i64>() {
                    Ok(id) => id,
                    Err(_) => {
                        let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                        return;
                    }
                },
                None => {
                    let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                    return;
                }
            };

            if let None = *current_user.lock().unwrap() {
                let _ = writeln!(&mut stdout.lock(), "{SET_BOARD_PROMPT}");
                return;
            }

            if backend::remove_entry(
                &create_interaction(&current_user, &cmd_arc),
                user_id
            ) {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "Removed {user_id}."
                );
            } else {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "{user_id} is not on the leaderboard."
                );
            }
        }
        "get" => {
            let usage_msg = "Usage: get <user_id>";

            if params.len() > 2 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            let user_id = match params.get(1) {
                Some(b) => match b.parse::<i64>() {
                    Ok(id) => id,
                    Err(_) => {
                        let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                        return;
                    }
                },
                None => {
                    let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                    return;
                }
            };

            if let None = *current_user.lock().unwrap() {
                let _ = writeln!(&mut stdout.lock(), "{SET_BOARD_PROMPT}");
                return;
            }

            match backend::get_points(&create_interaction(&current_user, &cmd_arc), &user_id) {
                Some(pts) => {
                    let _ = writeln!(&mut stdout.lock(), "User {user_id} has {pts} points.");
                }
                None => {
                    let _ = writeln!(&mut stdout.lock(), "User {user_id} is not on the board.");
                }
            }
        }
        "top" => {
            let usage_msg = "Usage: top <count>";

            if params.len() > 2 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            let count = match params.get(1) {
                Some(b) => match b.parse::<usize>() {
                    Ok(id) => id,
                    Err(_) => {
                        let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                        return;
                    }
                },
                None => {
                    let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                    return;
                }
            };

            if let None = *current_user.lock().unwrap() {
                let _ = writeln!(&mut stdout.lock(), "{SET_BOARD_PROMPT}");
                return;
            }

            for entry in backend::get_top(&create_interaction(&current_user, &cmd_arc), count).iter() {
                let _ = writeln!(&mut stdout.lock(), "{}:\t{}\t({} points)", entry.0 + 1, entry.1.key, entry.1.points);
            }
        }
        "bottom" => {
            let usage_msg = "Usage: bottom <count>";

            if params.len() > 2 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            let count = match params.get(1) {
                Some(b) => match b.parse::<usize>() {
                    Ok(id) => id,
                    Err(_) => {
                        let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                        return;
                    }
                },
                None => {
                    let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                    return;
                }
            };

            if let None = *current_user.lock().unwrap() {
                let _ = writeln!(&mut stdout.lock(), "{SET_BOARD_PROMPT}");
                return;
            }

            for entry in backend::get_bottom(&create_interaction(&current_user, &cmd_arc), count).iter().rev() {
                let _ = writeln!(&mut stdout.lock(), "{}:\t{}\t({} points)", entry.0 + 1, entry.1.key, entry.1.points);
            }
        }
        "rank" => {
            let usage_msg = "Usage: rank <user_id>";

            if params.len() > 2 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            let user_id = match params.get(1) {
                Some(b) => match b.parse::<i64>() {
                    Ok(id) => id,
                    Err(_) => {
                        let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                        return;
                    }
                },
                None => {
                    let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                    return;
                }
            };

            if let None = *current_user.lock().unwrap() {
                let _ = writeln!(&mut stdout.lock(), "{SET_BOARD_PROMPT}");
                return;
            }

            match backend::get_rank(&create_interaction(&current_user, &cmd_arc), &user_id) {
                Some(rank) => {
                    let _ = writeln!(
                        &mut stdout.lock(),
                        "User {user_id} is in #{rank} on the board."
                    );
                }
                None => {
                    let _ = writeln!(&mut stdout.lock(), "User {user_id} is not on the board.");
                }
            }
        }
        "size" => {
            let usage_msg = "Usage: size";

            if params.len() > 1 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            if let None = *current_user.lock().unwrap() {
                let _ = writeln!(&mut stdout.lock(), "{SET_BOARD_PROMPT}");
                return;
            }

            let _ = writeln!(
                &mut stdout.lock(),
                "Current board has {} entries.",
                backend::get_size(&create_interaction(&current_user, &cmd_arc))
            );
        }
        "clear" => {
            let usage_msg = "Usage: clear";

            if params.len() > 1 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            if let None = *current_user.lock().unwrap() {
                let _ = writeln!(&mut stdout.lock(), "{SET_BOARD_PROMPT}");
                return;
            }

            let _ = writeln!(
                &mut stdout.lock(),
                "Are you sure you want to clear this board and erase all associated data?"
            );
            if confirm_action() {
                let _ = writeln!(&mut stdout.lock(), "Clearing data...");
                backend::clear(&create_interaction(current_user, cmd_arc));
                let _ = writeln!(&mut stdout.lock(), "Cleared.");
            }
        }
        "populate" => {
            let usage_msg = "Usage: populate <count>";

            let count = match params.get(1) {
                Some(b) => match b.parse::<i64>() {
                    Ok(v) => v,
                    Err(_) => {
                        let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                        return;
                    }
                },
                None => {
                    let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                    return;
                }
            };

            if params.len() > 2 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            if let None = *current_user.lock().unwrap() {
                let _ = writeln!(&mut stdout.lock(), "{SET_BOARD_PROMPT}");
                return;
            }

            let interaction = create_interaction(current_user, cmd_arc);
            let mut binding = interaction.state.boards.lock().unwrap();
            let board = binding.get_mut(&interaction.user.board).unwrap();
            let empty = board.get_size() == 0;

            if !empty {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "Are you sure you want to populate this board with {count} dummy entries? This will clear all existing data on the current board."
                );
            }

            if empty || confirm_action() {
                let _ = writeln!(&mut stdout.lock(), "Clearing data...");

                board.clear();
                let _ = writeln!(&mut stdout.lock(), "Populating...");
                for i in 0..count {
                    let _ = board.update_entry(i + 1, i as f64);
                }
            }
        }
        "board" => {
            let usage_msg = "Usage: board <board_name>";

            if params.len() > 2 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            let board = match params.get(1) {
                Some(b) => b.to_string(),
                None => {
                    match current_user.lock().unwrap().clone() {
                        Some(usr) => {
                            let _ = writeln!(&mut stdout.lock(), "Current board: {}.", usr.board);
                        }
                        None => {
                            let _ = writeln!(&mut stdout.lock(), "{SET_BOARD_PROMPT}");
                        }
                    }
                    return;
                }
            };

            if !(cmd_arc.boards.lock().unwrap().contains_key(&board)) {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "Invalid board '{board}', does not exist."
                );
                return;
            }

            let mut state = current_user.lock().unwrap();
            let _ = std::mem::replace(
                &mut *state,
                Some(backend::User {
                    board: board,
                    write: true,
                }),
            );
        }
        "cap" => {
            let usage_msg = "Usage: cap <size>";

            if params.len() > 2 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            if current_user.lock().unwrap().is_none() {
                let _ = writeln!(&mut stdout.lock(), "{SET_BOARD_PROMPT}");
                return;
            }


            let cap = match params.get(1) {
                Some(&"-1") => {
                    let board_name = current_user.lock().unwrap().as_ref().unwrap().board.clone();
                    cmd_arc.rem_board_cap(&board_name);
                    let _ = writeln!(&mut stdout.lock(), "Board size cap removed.");
                    return;
                },
                Some(b) => match b.parse::<usize>() {
                    Ok(v) => v,
                    Err(_) => {
                        let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                        return;
                    }
                },
                None => {
                    let interaction = create_interaction(current_user, cmd_arc);
                    let mut binding = interaction.state.boards.lock().unwrap();
                    let board = binding.get_mut(&interaction.user.board).unwrap();

                    if let Some(cap) = board.get_size_cap() {
                        let _ = writeln!(&mut stdout.lock(), "Current size cap: {}.", cap);
                    } else {
                        let _ = writeln!(&mut stdout.lock(), "No size cap set.");
                    }
                    return;
                }
            };
            
            let board_name = current_user.lock().unwrap().as_ref().unwrap().board.clone();
            cmd_arc.set_board_cap(&board_name, cap);

            let interaction = create_interaction(current_user, cmd_arc);
            let mut binding = interaction.state.boards.lock().unwrap();
            let board = binding.get_mut(&interaction.user.board).unwrap();

            let proceed = board.get_size() <= cap;

            if !proceed {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "Do you want to trim off all {} entried beyond the cap? This data will not be retrievable.",
                    board.get_size() - cap
                );
            }

            if proceed || confirm_action() {
                let _ = writeln!(&mut stdout.lock(), "Trimming entries...");
                board.trim_after_cap();
            }
        }
        "new_board" => {
            let usage_msg = "Usage: new_board <name>";

            if params.len() > 2 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            let name = match params.get(1) {
                Some(b) => b,
                None => {
                    let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                    return;
                }
            };

            for c in name.chars() {
                let ascii = c.to_ascii_uppercase() as u8;
                if (ascii >= 65 && ascii <= 90) || (ascii == 95) || (ascii == 45) || (ascii == 46) || (ascii >= 48 && ascii <= 57) {
                    continue;
                }
                let _ = writeln!(&mut stdout.lock(), "Invalid character \"{}\" in board name.", c);
                return;
            }
            
            if cmd_arc.create_board(name.to_string()) {
                let _ = writeln!(&mut stdout.lock(), "Created board \"{}\".", name);
            } else {
                let _ = writeln!(&mut stdout.lock(), "Board already exists with name \"{}\".", name);
            }
        }
        "del_board" => {
            let usage_msg = "Usage: del_board";

            if params.len() > 1 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            if current_user.lock().unwrap().is_none() {
                let _ = writeln!(&mut stdout.lock(), "{SET_BOARD_PROMPT}");
                return;
            }

            let board_name = current_user.lock().unwrap().as_ref().unwrap().board.clone();
            let _ = writeln!(&mut stdout.lock(), "Are you want to delete board {} and all of its contents and data? This data cannot be retrieved.", board_name);

            if confirm_action() {
                let _ = writeln!(&mut stdout.lock(), "Deleting....");
                cmd_arc.delete_board(&board_name);

                let mut state = current_user.lock().unwrap();
                let _ = std::mem::replace(&mut *state, None);
            }
        }
        "trim" => {
            let usage_msg = "Usage: trim";

            if params.len() > 1 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            if current_user.lock().unwrap().is_none() {
                let _ = writeln!(&mut stdout.lock(), "{SET_BOARD_PROMPT}");
                return;
            }

            let interaction = create_interaction(current_user, cmd_arc);
            let mut binding = interaction.state.boards.lock().unwrap();
            let board = binding.get_mut(&interaction.user.board).unwrap();
            
            if board.get_size_cap().is_none() || board.get_size() <= board.get_size_cap().unwrap() {
                let _ = writeln!(&mut stdout.lock(), "Nothing to trim.");
            }

            let _ = writeln!(
                &mut stdout.lock(),
                "Do you want to trim off all {} entried beyond the cap? This data will not be retrievable.",
                board.get_size() - board.get_size_cap().unwrap()
            );

            if confirm_action() {
                let _ = writeln!(&mut stdout.lock(), "Trimming entries...");
                board.trim_after_cap();
            }
        }
        "boards" => {
            if params.len() > 1 {
                let _ = writeln!(&mut stdout.lock(), "Usage: save");
                return;
            }

            let mut ind = 0;
            for (name, _board) in cmd_arc.boards.lock().unwrap().iter() {
                let _ = writeln!(&mut stdout.lock(), "{ind}: {name}");
                ind += 1;
            }
            if ind == 0 {
                let _ = writeln!(&mut stdout.lock(), "No boards to display.");
            }
        }
        "help" => {
            let _ = writeln!(
                &mut stdout.lock(),
                "Commands:\n\
            help:\t\t\t\tThis.\n\
            \n\
            board:\t\t\t\tOutputs the current board. Board mut be set first using the board <board_name> command.\n\
            board <board_name>:\t\tSets the current board.\n\
            boards:\t\t\t\tGet a list of all leaderboards.\n\
            new_board <name>:\t\t\tCreate a new board with the given name.\n\
            del_board:\t\t\t\tDelete the current board along with all associated information.\n\
            \n\
            users:\t\t\t\tList all users on the current board.\n\
            new_user <api_key>:\t\tCreates a new user on the current board with the given API key.\n\
            delete_user <api_key>:\t\tDeletes the user on the current board with the given API key.\n\
            set_write <api_key> <y/n>:\t\t Sets whether or not a specific user on the current board has write permissions.\n\
            \n\
            get <user_id>:\t\t\tGets the number of points the specified user has on the current board.\n\
            rank <user_id>:\t\t\tGets the rank of the specified user in the leaderboard.\n\
            update <user_id> <points>:\tUpdates the specified user's points on the current board.\n\
            remove <user_id>:\t\t\tRemoves a specific user from the leaderboard.\n\
            \n\
            top <count>:\t\t\t: Gets the top <count> users in the leaderboard.\n\
            bottom <count>:\t\t\t: Gets the bottom <count> users in the leaderboard.\n\
            size:\t\t\t\tReturns the number of entries in the current leaderboard\n\
            \n\
            populate <count>:\t\t\tFill the current board with <count> dummy entries, good for testing scalability. Overwrites ALL existing data.\n\
            clear:\t\t\t\tEntirely clears the current leaderboard, erasing all data\n\
            \n\
            cap:\t\t\t\tGet the size cap of the current leaderboard.\n\
            cap <size>:\t\t\tSet the size cap of the current leaderboard. Set to -1 to remove cap.\n\
            trim:\t\t\t\tTrims off elements from the end of the current leaderboard until it's size is under the cap.\n\
            \n\
            save:\t\t\t\tSaves all boards to file.\n\
            Ctrl+C:\t\t\t\tSave all boards, stop the program, and shut down the server."
            );
        }
        _other => {
            let _ = writeln!(&mut stdout.lock(), "Invalid command '{cmd}'");
        }
    }
}

pub fn put_cli_prompt() {
    let stdout = io::stdout();
    let _ = write!(&mut stdout.lock(), "> ");
    let _ = stdout.lock().flush();
}

pub fn exec_cli(cmd_arc: Arc<AppState>, cmd_saves_path: PathBuf) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut s;
    let _ = writeln!(
        &mut stdout.lock(),
        "CLI launched, type 'help' for a list of commands."
    );
    let current_user: Mutex<Option<User>> = Mutex::new(None);

    loop {
        put_cli_prompt();

        s = "".to_string();
        let res = stdin.lock().read_line(&mut s);
        if res.is_err() {
            let _ = writeln!(
                &mut stdout.lock(),
                "Failed to read input from CLI.\n{}",
                res.unwrap_err()
            );
            break;
        }

        let params: Vec<&str> = s.trim().split(" ").collect();

        if params.len() == 0 {
            continue;
        }

        execute_command(params, &cmd_arc, &cmd_saves_path, &current_user);
    }
}
