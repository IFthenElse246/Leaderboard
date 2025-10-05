use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};

use rand::distr::{Distribution, Uniform};

use crate::{
    app_state::AppState,
    backend::{self, Interaction, User},
    board::Board,
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
                    }
                    false => {
                        let _ = writeln!(
                            &mut stdout.lock(),
                            "Added {user_id} to have {points} points."
                        );
                    }
                },
                Err(v) => {
                    let _ = writeln!(&mut stdout.lock(), "Failed to add {user_id}:\n{v}.");
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

            if backend::remove_entry(&create_interaction(&current_user, &cmd_arc), user_id).is_some() {
                let _ = writeln!(&mut stdout.lock(), "Removed {user_id}.");
            } else {
                let _ = writeln!(&mut stdout.lock(), "{user_id} is not on the leaderboard.");
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

            for entry in
                backend::get_top(&create_interaction(&current_user, &cmd_arc), count, true).iter()
            {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "{}:\t{}\t({} points)",
                    entry.0,
                    entry.1.key,
                    entry.1.points
                );
            }
        }
        "after" => {
            let usage_msg = "Usage: after <user_id> <count>";

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

            let count = match params.get(2) {
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

            let result = match backend::get_after(
                &create_interaction(&current_user, &cmd_arc),
                &user_id,
                count,
            ) {
                Some(v) => v,
                None => {
                    let _ = writeln!(&mut stdout.lock(), "User id {user_id} is not in the board.");
                    return;
                }
            };

            for entry in result.iter() {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "{}:\t{}\t({} points)",
                    entry.0,
                    entry.1.key,
                    entry.1.points
                );
            }
        }
        "before" => {
            let usage_msg = "Usage: before <user_id> <count>";

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

            let count = match params.get(2) {
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

            let result = match backend::get_before(
                &create_interaction(&current_user, &cmd_arc),
                &user_id,
                count,
            ) {
                Some(v) => v,
                None => {
                    let _ = writeln!(&mut stdout.lock(), "User id {user_id} is not in the board.");
                    return;
                }
            };

            for entry in result.iter().rev() {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "{}:\t{}\t({} points)",
                    entry.0,
                    entry.1.key,
                    entry.1.points
                );
            }
        }
        "around" => {
            let usage_msg = "Usage: around <user_id> <before> <after>";

            if params.len() > 4 {
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

            let before = match params.get(2) {
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

            let after = match params.get(3) {
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

            let result = match backend::get_around(
                &create_interaction(&current_user, &cmd_arc),
                &user_id,
                before,
                after,
            ) {
                Some(v) => v,
                None => {
                    let _ = writeln!(&mut stdout.lock(), "User id {user_id} is not in the board.");
                    return;
                }
            };

            for entry in result.iter() {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "{}:\t{}\t({} points)",
                    entry.0,
                    entry.1.key,
                    entry.1.points
                );
            }
        }
        "range" => {
            let usage_msg = "Usage: range <start> <end>";

            if params.len() > 3 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            let start = match params.get(1) {
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

            let end = match params.get(2) {
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

            let result =
                backend::get_range(&create_interaction(&current_user, &cmd_arc), start, end);

            if result.len() == 0 {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "There are no entries with ranks between {start} and {end}."
                );
                return;
            }

            for entry in result.iter() {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "{}:\t{}\t({} points)",
                    entry.0,
                    entry.1.key,
                    entry.1.points
                );
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

            for entry in backend::get_bottom(&create_interaction(&current_user, &cmd_arc), count, true)
                .iter()
                .rev()
            {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "{}:\t{}\t({} points)",
                    entry.0,
                    entry.1.key,
                    entry.1.points
                );
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
        "at_rank" => {
            let usage_msg = "Usage: at_rank <rank>";

            if params.len() > 2 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            let rank = match params.get(1) {
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

            match backend::at_rank(&create_interaction(&current_user, &cmd_arc), rank.clone()) {
                Some(entry) => {
                    let _ = writeln!(
                        &mut stdout.lock(),
                        "The entry at rank #{}:\n{}\t({} points)",
                        rank,
                        entry.key,
                        entry.points
                    );
                }
                None => {
                    let _ = writeln!(&mut stdout.lock(), "No entry with rank {rank}.");
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
                }
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
                Some(&"") | None => {
                    let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                    return;
                }
                Some(b) => b,
            };

            for c in name.chars() {
                let ascii = c.to_ascii_uppercase() as u8;
                if (ascii >= 65 && ascii <= 90)
                    || (ascii == 95)
                    || (ascii == 45)
                    || (ascii == 46)
                    || (ascii >= 48 && ascii <= 57)
                {
                    continue;
                }
                let _ = writeln!(
                    &mut stdout.lock(),
                    "Invalid character \"{}\" in board name.",
                    c
                );
                return;
            }

            if cmd_arc.create_board(name.to_string()) {
                let _ = writeln!(&mut stdout.lock(), "Created board \"{}\".", name);
            } else {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "Board already exists with name \"{}\".",
                    name
                );
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
            let _ = writeln!(
                &mut stdout.lock(),
                "Are you want to delete board {} and all of its contents and data? This data cannot be retrieved.",
                board_name
            );

            if confirm_action() {
                let _ = writeln!(&mut stdout.lock(), "Deleting....");
                cmd_arc.delete_board(&board_name);

                let mut state = current_user.lock().unwrap();
                let _ = std::mem::replace(&mut *state, None);
            }
        }
        "keys" => {
            let usage_msg = "Usage: keys";

            if params.len() > 1 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            let keys = cmd_arc.api_keys.lock().unwrap();

            if current_user.lock().unwrap().is_none() {
                let _ = writeln!(&mut stdout.lock(), "{SET_BOARD_PROMPT}");
                return;
            }

            let board_name = current_user.lock().unwrap().as_ref().unwrap().board.clone();
            let mut ind = 0;

            for (key, user) in keys.iter() {
                if user.board != board_name {
                    continue;
                }
                let _ = writeln!(
                    &mut stdout.lock(),
                    "{}:\t\t{}",
                    key,
                    if user.write { "write" } else { "read" }
                );
                ind += 1;
            }

            if ind == 0 {
                let _ = writeln!(&mut stdout.lock(), "No keys to display.");
            }
        }
        "all_keys" => {
            let usage_msg = "Usage: all_keys";

            if params.len() > 1 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            let keys = cmd_arc.api_keys.lock().unwrap();

            if keys.len() == 0 {
                let _ = writeln!(&mut stdout.lock(), "No keys to display.");
                return;
            }
            for (key, user) in keys.iter() {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "{}:\t\t{}\t{}",
                    key,
                    user.board,
                    if user.write { "write" } else { "read" }
                );
            }
        }
        "new_key" => {
            let usage_msg = "Usage: new_key <api_key> <write y/n>";

            if params.len() > 3 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            if current_user.lock().unwrap().is_none() {
                let _ = writeln!(&mut stdout.lock(), "{SET_BOARD_PROMPT}");
                return;
            }

            let key = match params.get(1) {
                Some(&"") | None => {
                    let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                    return;
                }
                Some(b) => b,
            };

            let w = match params.get(2) {
                Some(&"y") | Some(&"Y") => true,
                Some(&"n") | Some(&"N") => false,
                _v => {
                    let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                    return;
                }
            };

            let board_name = current_user.lock().unwrap().as_ref().unwrap().board.clone();

            if cmd_arc.create_key(key.to_string(), board_name.clone(), w) {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "Added API Key {key} to board {board_name} {} write access.",
                    if w { "with" } else { "without" }
                );
            } else {
                let _ = writeln!(&mut stdout.lock(), "API Key {key} already in use!");
            }
        }
        "del_key" => {
            let usage_msg = "Usage: del_key <api_key>";

            if params.len() > 2 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            let key = match params.get(1) {
                Some(&"") | None => {
                    let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                    return;
                }
                Some(b) => b,
            };

            if cmd_arc.delete_key(&key.to_string()) {
                let _ = writeln!(&mut stdout.lock(), "Deleted API Key {key}.");
            } else {
                let _ = writeln!(&mut stdout.lock(), "API Key {key} does not exist!");
            }
        }
        "set_write" => {
            let usage_msg = "Usage: set_write <api_key> <y/n>";

            if params.len() > 3 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            let key = match params.get(1) {
                Some(&"") | None => {
                    let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                    return;
                }
                Some(b) => b,
            };

            let write = match params.get(2) {
                Some(&"y") | Some(&"Y") => true,
                Some(&"n") | Some(&"N") => false,
                _v => {
                    let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                    return;
                }
            };

            if cmd_arc.set_key_write_perms(&key.to_string(), write) {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "Set API Key {key} to {} write access on its board.",
                    if write { "have" } else { "not have" }
                );
            } else {
                let _ = writeln!(&mut stdout.lock(), "No API Key {key}!");
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
        "stress_test" => {
            let usage_msg = "Usage: stress_test <board_size>";

            if params.len() > 2 {
                let _ = writeln!(&mut stdout.lock(), "{usage_msg}");
                return;
            }

            let size = match params.get(1) {
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
            let usize_size: usize = match size.try_into() {
                Ok(val) => val,
                Err(_v) => {
                    let _ = writeln!(&mut stdout.lock(), "Invalid size for stress test.");
                    return;
                }
            };

            if size <= 0 {
                let _ = writeln!(&mut stdout.lock(), "Invalid size for stress test.");
                return;
            }

            let save_locker = cmd_arc.save_locker.lock().unwrap();

            if let None = *current_user.lock().unwrap() {
                let _ = writeln!(&mut stdout.lock(), "{SET_BOARD_PROMPT}");
                return;
            }
            let board_name = current_user.lock().unwrap().as_ref().unwrap().board.clone();

            let interaction = create_interaction(current_user, cmd_arc);
            let mut binding = interaction.state.boards.lock().unwrap();
            let board = binding.get_mut(&interaction.user.board).unwrap();
            let empty = board.get_size() == 0;

            if !empty {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "Are you sure you want to COMPLETELY CLEAR this board to use it for a stress test? It is strongly advised to make a new temporary board for stress testing."
                );

                if confirm_action() {
                    let _ = writeln!(&mut stdout.lock(), "Clearing existing data...");

                    board.clear();
                }
            }

            let _ = drop(binding);

            let _ = writeln!(&mut stdout.lock(), "Capping entries...");
            cmd_arc.set_board_cap(&board_name, usize_size);

            let mut binding = interaction.state.boards.lock().unwrap();
            let board = binding.get_mut(&interaction.user.board).unwrap();

            let _ = writeln!(&mut stdout.lock(), "Populating with dummy entries...");
            for i in 0..size {
                let _ = board.update_entry(i, i as f64);
            }

            let _ = drop(binding);

            let mut snapshot = None;

            if !cmd_arc.lock_save {
                snapshot = Some(
                    cmd_arc
                        .boards
                        .lock()
                        .unwrap()
                        .get(&board_name)
                        .unwrap()
                        .get_map_snapshot(),
                ); // just so things are slowed down
            }

            let _ = writeln!(&mut stdout.lock(), "Performing update test...");
            let mut start = Instant::now();
            let mut elapsed;
            let mut num_writes = 0;
            let mut rng = rand::rng();
            let id_range = Uniform::try_from(i64::MIN..=i64::MAX).unwrap();
            let val_range = Uniform::try_from(0..size).unwrap();

            loop {
                elapsed = start.elapsed();
                if elapsed.as_millis() >= 1000 {
                    break;
                }

                let interaction = create_interaction(current_user, cmd_arc);
                let target_id = id_range.sample(&mut rng);
                let target_value = val_range.sample(&mut rng);
                let _ = backend::update_entry(&interaction, target_id, target_value as f64);

                num_writes += 1;
            }

            let _ = writeln!(
                &mut stdout.lock(),
                "Made {num_writes} update operations in {} ms.",
                start.elapsed().as_millis()
            );

            let write_time = start.elapsed().as_secs_f64() / (num_writes as f64);

            let _ = writeln!(&mut stdout.lock(), "Preparing...");

            let ids: Vec<i64> = cmd_arc
                .boards
                .lock()
                .unwrap()
                .get(&board_name)
                .unwrap()
                .get_ids();
            let ind_range = Uniform::try_from(0..ids.len()).unwrap();

            let _ = writeln!(&mut stdout.lock(), "Performing retrieval around test...");

            start = Instant::now();
            let mut num_reads = 0;
            loop {
                elapsed = start.elapsed();
                if elapsed.as_millis() >= 1000 {
                    break;
                }

                let interaction = create_interaction(current_user, cmd_arc);
                let target_id = ids.get(ind_range.sample(&mut rng)).unwrap();

                let _ = backend::get_around(&interaction, target_id, 25, 25);

                num_reads += 1;
            }

            let _ = writeln!(
                &mut stdout.lock(),
                "Made {num_reads} retrieval around operations in {} ms.",
                start.elapsed().as_millis()
            );

            let read_time = start.elapsed().as_secs_f64() / (num_reads as f64);

            let _ = writeln!(&mut stdout.lock(), "Performing rank retrieval test...");

            start = Instant::now();
            let mut num_ranks = 0;

            loop {
                elapsed = start.elapsed();
                if elapsed.as_millis() >= 1000 {
                    break;
                }

                let interaction = create_interaction(current_user, cmd_arc);
                let target_id = ids.get(ind_range.sample(&mut rng)).unwrap();

                let _ = backend::get_rank(&interaction, target_id);

                num_ranks += 1;
            }

            let _ = writeln!(
                &mut stdout.lock(),
                "Made {num_ranks} rank retrieval operations in {} ms.",
                start.elapsed().as_millis()
            );

            let rank_time = start.elapsed().as_secs_f64() / (num_ranks as f64);

            let _ = writeln!(&mut stdout.lock(), "Getting top 50 entries...");
            start = Instant::now();

            let interaction = create_interaction(current_user, cmd_arc);
            let _ = backend::get_top(&interaction, 50, true);

            let top_time = start.elapsed().as_secs_f64();

            let _ = writeln!(&mut stdout.lock(), "Getting bottom 50 entries...");
            start = Instant::now();

            let interaction = create_interaction(current_user, cmd_arc);
            let _ = backend::get_bottom(&interaction, 50, true);

            let bottom_time = start.elapsed().as_secs_f64();

            let _ = writeln!(&mut stdout.lock(), "Preparing to write to file...");
            start = Instant::now();

            let _ = drop(snapshot);

            let write_start_time = start.elapsed().as_secs_f64();

            let _ = writeln!(&mut stdout.lock(), "Writing to file...");
            start = Instant::now();

            let temp_path = cmd_arc.saves_path.join(format!("{board_name}_saving.test"));

            let mut snapshot_clone_time = None;
            let mut snapshot_clone = None;

            if !cmd_arc.lock_save {
                let snapshot = cmd_arc
                    .boards
                    .lock()
                    .unwrap()
                    .get(&board_name)
                    .unwrap()
                    .get_map_snapshot();

                snapshot_clone = Some(snapshot.get_lock().clone());
                snapshot_clone_time = Some(start.elapsed().as_secs_f64());
                start = Instant::now();

                let _ = drop(snapshot);
            }

            match File::create(&temp_path) {
                Ok(handle) => {
                    let mut buf_writer = BufWriter::new(handle);

                    let result;

                    if cmd_arc.lock_save {
                        let boards = cmd_arc.boards.lock().unwrap();
                        let board = boards.get(&board_name);

                        result = bincode::encode_into_std_write(
                            &board,
                            &mut buf_writer,
                            bincode::config::standard(),
                        );

                        let _ = drop(boards);
                    } else {
                        start = Instant::now();

                        result = bincode::encode_into_std_write(
                            &snapshot_clone.unwrap(),
                            &mut buf_writer,
                            bincode::config::standard(),
                        );
                    }

                    if let Err(e) = result {
                        let _ = writeln!(
                            &mut io::stderr().lock(),
                            "Failed to serialize leaderboard:\n{e}"
                        );
                    }
                }
                Err(err) => {
                    let _ = writeln!(
                        &mut io::stderr().lock(),
                        "Failed to open temp file to save board.\n{}",
                        err
                    );
                }
            };

            let write_file_time = start.elapsed().as_secs_f64();

            let _ = writeln!(&mut stdout.lock(), "Preparing to read from file...");
            
            let mut binding = cmd_arc.boards.lock().unwrap();
            let board = binding.get_mut(&interaction.user.board).unwrap();
            board.clear();

            let _ = drop(binding);

            let _ = writeln!(&mut stdout.lock(), "Reading from file...");
            start = Instant::now();

            let board: Board<i64, f64>;

            match File::open(temp_path.clone()) {
                Err(e) => {
                    let _ = writeln!(
                        &mut io::stderr().lock(),
                        "Failed to read file for leaderboard:\n{e}",
                    );
                    board = Board::new();
                }
                Ok(save_file) => {
                    let mut buf_reader = BufReader::new(save_file);

                    board = match bincode::decode_from_std_read(
                        &mut buf_reader,
                        bincode::config::standard(),
                    ) {
                        Err(e) => {
                            let _ = writeln!(
                                &mut io::stderr().lock(),
                                "Failed to parse file for leaderboard:\n{e}",
                            );
                            Board::new()
                        }
                        Ok(b) => b,
                    };
                }
            };

            let read_file_time = start.elapsed().as_secs_f64();

            let _ = writeln!(&mut stdout.lock(), "Cleaning up...");

            let _ = drop(board);

            if temp_path.exists() {
                let _ = std::fs::remove_file(temp_path);
            }

            if cmd_arc.lock_save {
                let _ = writeln!(
                    &mut stdout.lock(),
                    "\nRESULTS:\n\
                In a board with {size} entries...\n\
                Updating an user's points takes roughly {:.4} ms.\n\
                Retrieving the 25 entries before and after (total 51 entries) a user takes roughly {:.4} ms.\n\
                Retrieving the rank of a user takes roughly {:.4} ms.\n\
                Getting the top 50 entries takes roughly {:.4} ms.\n\
                Getting the bottom 50 entries takes roughly {:.4} ms.\n\
                Saving to file will take {:.4} seconds. All operations on the board will halt while this is in progress.\n\
                Reading from file will take {:.4} seconds.\n\
                *Please note that these tests do not factor things in like parsing HTTP requests and thus should not be trusted entirely. These lengths were calculated by directly performing these operations and should only serve as a benchmark or rough reference. The read write operation tests should be completely accurate in terms of length, however.",
                    write_time * 1000.0,
                    read_time * 1000.0,
                    rank_time * 1000.0,
                    top_time * 1000.0,
                    bottom_time * 1000.0,
                    write_file_time,
                    read_file_time
                );
            } else {
                let merge_time = write_start_time / (write_time * (num_writes as f64)) * snapshot_clone_time.unwrap();
                let _ = writeln!(
                    &mut stdout.lock(),
                    "\nRESULTS:\n\
                In a board with {size} entries...\n\
                Updating an user's points takes roughly {:.4} ms.\n\
                Retrieving the 25 entries before and after (total 51 entries) a user takes roughly {:.4} ms.\n\
                Retrieving the rank of a user takes roughly {:.4} ms.\n\
                Getting the top 50 entries takes roughly {:.4} ms.\n\
                Getting the bottom 50 entries takes roughly {:.4} ms.\n\
                Preparing to save to file will take {:.4} seconds.\n\
                Before writing to file, everything will slow for ROUGHLY {:.4} seconds.\n\
                Writing to file will take {:.4} seconds in total.\n\
                Reading from file will take {:.4} seconds.\n\
                *Please note that these tests do not factor things in like parsing HTTP requests and thus should not be trusted entirely. These lengths were calculated by directly performing these operations and should only serve as a benchmark or rough reference. The read write operation tests should be completely accurate in terms of length, however.",
                    write_time * 1000.0,
                    read_time * 1000.0,
                    rank_time * 1000.0,
                    top_time * 1000.0,
                    bottom_time * 1000.0,
                    snapshot_clone_time.unwrap(),
                    merge_time,
                    write_file_time,
                    read_file_time
                );
            }

            let _ = drop(save_locker);
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
            new_board <name>:\t\tCreate a new board with the given name.\n\
            del_board:\t\t\tDelete the current board along with all associated information.\n\
            \n\
            keys:\t\t\t\tList all API Keys on the current board.\n\
            all_keys:\t\t\tList all API Keys on all boards.\n\
            new_key <api_key> <write y/n>:\tCreates a new API Key on the current board with specified permissions.\n\
            del_key <api_key>:\t\tRemoves the specified API Key from the current board.\n\
            set_write <api_key> <y/n>:\tSets whether or not a specific API Key on the current board has write permissions.\n\
            \n\
            get <user_id>:\t\t\tGets the number of points the specified user has on the current board.\n\
            rank <user_id>:\t\t\tGets the rank of the specified user in the leaderboard.\n\
            at_rank <rank>:\t\t\tGets the entry of the leaderboard at the specified rank.\n\
            update <user_id> <points>:\tUpdates the specified user's points on the current board.\n\
            remove <user_id>:\t\tRemoves a specific user from the leaderboard.\n\
            \n\
            top <count>:\t\t\tGets the top <count> users in the leaderboard.\n\
            bottom <count>:\t\t\tGets the bottom <count> users in the leaderboard.\n\
            after <user_id> <count>:\tGets the <count> entries after the given user in the board.\n\
            before <user_id> <count>:\tGets the <count> before after the given user in the board.\n\
            around <user_id> <before> <after>:\tGets the entries around the given user.\n\
            range <start> <end>:\t\tGets the entries with ranks between <start> and <end>\n\
            \n\
            size:\t\t\t\tReturns the number of entries in the current leaderboard\n\
            clear:\t\t\t\tEntirely clears the current leaderboard, erasing all data\n\
            \n\
            populate <count>:\t\tFill the current board with <count> dummy entries, good for testing scalability. Overwrites ALL existing data.\n\
            stress_test <board_size>:\tOutputs the number of operations that can be made in a second on a dummy board with specified size.\n\
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
