extern crate fs2;

use crate::backend::{ActionType, AppState, Interaction, User};
use fs2::FileExt;
use rocket::{
    State,
    fairing::AdHoc,
    http::Status,
    request::{FromRequest, Outcome, Request},
    tokio,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{self, BufRead, Read, Write},
    sync::Arc,
};

mod backend;
mod util;

#[macro_use]
extern crate rocket;

#[derive(Debug)]
pub enum RequestError {
    InvalidResult,
}

#[post("/update", format = "json", data = "<data>")]
fn test(interaction: Interaction, data: String) -> Result<String, Status> {
    interaction.state.actions[&ActionType::Update](interaction, data)
}

#[derive(Debug)]
pub enum ApiKeyError {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Interaction<'r> {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let state = req.guard::<&State<Arc<AppState>>>().await;
        if let Some(state) = state.succeeded() {
            let keys = state.api_keys.lock().unwrap();

            return match req.headers().get_one("x-api-key") {
                None => Outcome::Error((Status::BadRequest, ApiKeyError::Missing)),
                Some(key) if keys.contains_key(key) => Outcome::Success(Interaction {
                    user: keys.get(key).unwrap().clone(),
                    state: state,
                }),
                Some(_) => Outcome::Error((Status::Unauthorized, ApiKeyError::Invalid)),
            };
        }

        return Outcome::Error((Status::InternalServerError, ApiKeyError::Invalid));
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let executable_path = std::env::current_exe().unwrap();
    let main_path = executable_path.parent().unwrap();
    let file: std::fs::File = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(main_path.join("config.json"))
        .expect("Failed to open config file.");
    let boards_file: std::fs::File = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(main_path.join("boards.json"))
        .expect("Failed to open boards file.");

    boards_file
        .try_lock_exclusive()
        .expect("Boards file in use by another program, could not lock.");

    let state = AppState::new(&file, boards_file);

    drop(file);

    let port_arc = Arc::new(state);
    let state_arc = port_arc.clone();
    let loop_arc = port_arc.clone();
    let port = port_arc.port;

    let r = rocket::build()
        .configure(rocket::Config::figment().merge(("port", port)))
        .manage(state_arc)
        .mount("/", routes![test])
        .attach(AdHoc::on_liftoff("Save Loop", |_r| {
            Box::pin(async move {
                tokio::spawn(async move {
                    backend::save_loop(loop_arc).await;
                });
            })
        }))
        .attach(AdHoc::on_liftoff("CLI", |_r| {
            Box::pin(async move {
                tokio::spawn(async move {
                    let mut stdin = io::stdin().lock();
                    let stdout = io::stdout();
                    let mut s = String::new();
                    loop {
                        let _ = write!(&mut stdout.lock(), "> ");
                        stdout.lock().flush().unwrap();
                        s = "".to_string();
                        let res = stdin.read_line(&mut s);
                        if res.is_err() {
                            break;
                        }

                        let params: Vec<&str> = s.trim().split(" ").collect();

                        if params.len() == 0 {
                            continue;
                        }

                        let _ = writeln!(&mut stdout.lock(), "Cmd: {0}", params.get(0).unwrap());
                        
                    }
                });
            })
        }))
        .ignite()
        .await?;

    r.launch().await?;

    Ok(())
}
