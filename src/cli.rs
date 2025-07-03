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
                Some(b) => match b.parse::<u64>() {
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

            if backend::update_entry(
                &create_interaction(&current_user, &cmd_arc),
                user_id,
                points,
            ) {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "Updated {user_id} to have {points} points."
                );
            } else {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "Added {user_id} to have {points} points."
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
                Some(b) => match b.parse::<u64>() {
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

            match backend::get_points(&create_interaction(&current_user, &cmd_arc), user_id) {
                Some(pts) => {
                    let _ = writeln!(&mut stdout.lock(), "User {user_id} has {pts} points.");
                }
                None => {
                    let _ = writeln!(&mut stdout.lock(), "User {user_id} is not on the board.");
                }
            }
        }
        "rank" => {
            let usage_msg = "Usage: rank <user_id>";

            if params.len() > 2 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            let user_id = match params.get(1) {
                Some(b) => match b.parse::<u64>() {
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

            match backend::get_rank(&create_interaction(&current_user, &cmd_arc), user_id) {
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
                Some(b) => match b.parse::<u64>() {
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
                    board.update_entry(i + 1, i as f64);
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
                            let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
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
        }
        "help" => {
            let _ = writeln!(
                &mut stdout.lock(),
                "Commands:\n\
            help:\t\t\t\tThis.\n\
            save:\t\t\t\tSaves all boards to file.\n\
            board:\t\t\t\tOutputs the current board. Board mut be set first using the board <board_name> command.\n\
            board <board_name>:\t\tSets the current board.\n\
            get <user_id>:\t\t\tGets the number of points the specified user has on the current board.\n\
            rank <user_id>:\t\t\tGets the rank of the specified user in the leaderboard.\n\
            update <user_id> <points>:\tUpdates the specified user's points on the current board.\n\
            populate <count>:\t\t\tFill the current board with <count> dummy entries, good for testing scalability. Overwrites ALL existing data.\n\
            size:\t\t\t\tReturns the number of entries in the current leaderboard\n\
            clear:\t\t\t\tEntirely clears the current leaderboard, erasing all data\n\
            boards:\t\t\t\tGet a list of all boards.\n\
            Ctrl+C:\t\t\t\tStop the program and shut down the server."
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
