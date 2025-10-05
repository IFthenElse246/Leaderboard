extern crate fs2;

use crate::app_state::AppState;
use crate::backend::*;
use fs2::FileExt;
use rocket::tokio::fs;
use rocket::{
    State,
    fairing::AdHoc,
    http::Status,
    request::{FromRequest, Outcome, Request},
    tokio,
};
use serde::{Deserialize, Serialize};
use std::io::{Write, stdout};
use std::{fs::OpenOptions, sync::Arc};

pub mod app_state;
pub mod backend;
pub mod board;
mod cli;
pub mod util;

#[macro_use]
extern crate rocket;

pub type Key = i64;
pub type Val = f64;

#[derive(Debug)]
pub enum RequestError {
    InvalidResult,
}

#[post("/update", format = "json", data = "<data>")]
fn update(interaction: Interaction, data: String) -> Result<String, Status> {
    execute_update(&interaction, data)
}

#[post("/remove", format = "json", data = "<data>")]
fn remove(interaction: Interaction, data: String) -> Result<String, Status> {
    execute_remove(&interaction, data)
}

#[post("/get", format = "json", data = "<data>")]
fn get(interaction: Interaction, data: String) -> Result<String, Status> {
    execute_get(&interaction, data)
}

#[post("/info", format = "json", data = "<data>")]
fn info(interaction: Interaction, data: String) -> Result<String, Status> {
    execute_info(&interaction, data)
}

#[post("/board", format = "json", data = "<data>")]
fn board_info(interaction: Interaction, data: String) -> Result<String, Status> {
    execute_board(&interaction, data)
}

#[post("/atrank", format = "json", data = "<data>")]
fn at_rank(interaction: Interaction, data: String) -> Result<String, Status> {
    execute_at_rank(&interaction, data)
}

#[post("/top", format = "json", data = "<data>")]
fn top(interaction: Interaction, data: String) -> Result<String, Status> {
    execute_top(&interaction, data)
}

#[post("/bottom", format = "json", data = "<data>")]
fn bottom(interaction: Interaction, data: String) -> Result<String, Status> {
    execute_bottom(&interaction, data)
}

#[post("/after", format = "json", data = "<data>")]
fn after(interaction: Interaction, data: String) -> Result<String, Status> {
    execute_after(&interaction, data)
}

#[post("/before", format = "json", data = "<data>")]
fn before(interaction: Interaction, data: String) -> Result<String, Status> {
    execute_before(&interaction, data)
}

#[post("/around", format = "json", data = "<data>")]
fn around(interaction: Interaction, data: String) -> Result<String, Status> {
    execute_around(&interaction, data)
}

#[post("/range", format = "json", data = "<data>")]
fn range(interaction: Interaction, data: String) -> Result<String, Status> {
    execute_range(&interaction, data)
}

#[derive(Serialize, Deserialize)]
struct BatchRequest {
    req_type: backend::ActionType,
    payload: String,
}

#[post("/batch", format = "json", data = "<data>")]
fn batch(interaction: Interaction, data: String) -> Result<String, Status> {
    let json_res = serde_json::from_str::<Vec<BatchRequest>>(data.as_str());
    if json_res.is_err() {
        return Err(Status::BadRequest);
    }

    let json = json_res.unwrap();

    let mut result = Vec::with_capacity(json.len());

    for req in json {
        result.push(execute_action(req.req_type, &interaction, req.payload)?);
    }

    return Ok(serde_json::to_string(&result).unwrap());
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
        .mount(
            "/",
            routes![
                update, remove, get, info, board_info, at_rank, top, bottom, after, before, around,
                range, batch
            ],
        )
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
