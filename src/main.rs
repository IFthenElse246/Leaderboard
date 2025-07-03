extern crate fs2;

use crate::app_state::AppState;
use crate::backend::{ActionType, Interaction, execute_action};
use fs2::FileExt;
use rocket::tokio::fs;
use rocket::{
    State,
    fairing::AdHoc,
    http::Status,
    request::{FromRequest, Outcome, Request},
    tokio,
};
use std::io::{Write, stdout};
use std::{fs::OpenOptions, sync::Arc};

pub mod app_state;
pub mod backend;
pub mod board;
mod cli;
pub mod tree;
pub mod tree_node;
pub mod util;

#[macro_use]
extern crate rocket;

#[derive(Debug)]
pub enum RequestError {
    InvalidResult,
}

#[post("/update", format = "json", data = "<data>")]
fn test(interaction: Interaction, data: String) -> Result<String, Status> {
    execute_action(ActionType::Update, &interaction, data)
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
    let main_path = executable_path.parent().unwrap().to_path_buf();
    let saves_path = main_path.join("saves").to_path_buf();
    let cmd_saves_path = saves_path.clone();
    let shutdown_saves_path = saves_path.clone();
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

    if !saves_path.exists() {
        fs::create_dir(&saves_path)
            .await
            .expect("Could not create saves folder.");
    }

    boards_file
        .try_lock_exclusive()
        .expect("Boards file in use by another program, could not lock.");

    let state = AppState::new(&file, boards_file, &saves_path);

    drop(file);

    let port_arc = Arc::new(state);
    let state_arc = port_arc.clone();
    let loop_arc = port_arc.clone();
    let cmd_arc = port_arc.clone();
    let shutdown_arc = port_arc.clone();
    let port = port_arc.port;

    let r = rocket::build()
        .configure(rocket::Config::figment().merge(("port", port)))
        .manage(state_arc)
        .mount("/", routes![test])
        .attach(AdHoc::on_liftoff("Save Loop", |_r| {
            Box::pin(async move {
                tokio::spawn(async move {
                    backend::save_loop(loop_arc, &saves_path).await;
                });
            })
        }))
        .attach(AdHoc::on_liftoff("CLI", |_r| {
            Box::pin(async move {
                tokio::spawn(async move {
                    cli::exec_cli(cmd_arc, cmd_saves_path);
                });
            })
        }))
        .ignite()
        .await?;

    r.launch().await?;

    let _ = writeln!(stdout().lock(), "Performing safe shutdown...");
    backend::save(&shutdown_arc, &shutdown_saves_path);

    Ok(())
}
