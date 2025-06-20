extern crate fs2;

use rocket::{
    fairing::AdHoc, http::Status, request::{FromRequest, Outcome, Request}, tokio, State
};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use crate::backend::{AppState, User};

mod backend;
mod util;

#[macro_use]
extern crate rocket;

#[derive(Serialize, Deserialize)]
struct TestRequest {
    key: String
}

#[derive(Debug)]
pub enum RequestError {
    InvalidResult
}

#[post("/test", format="json", data="<data>")]
fn test(user: User, data: String) -> Result<String, Status> {
    let request = serde_json::from_str::<TestRequest>(data.as_str());

    if let Err(_) = request {
        return Result::Err(Status::BadRequest);
    }
    
    return match serde_json::to_string::<TestRequest>(&request.unwrap()) {
        Ok(str) => Result::Ok(str),
        Err(_) => Result::Err(Status::InternalServerError)
    };
}

#[derive(Debug)]
pub enum ApiKeyError {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let state = req.guard::<&State<AppState>>().await;
        if let Some(state) = state.succeeded() {
            let keys = state.api_keys.lock().unwrap();

            return match req.headers().get_one("x-api-key") {
                None => Outcome::Error((Status::BadRequest, ApiKeyError::Missing)),
                Some(key) if keys.contains_key(key) => {
                    Outcome::Success(keys.get(key).unwrap().clone())
                }
                Some(_) => Outcome::Error((Status::Unauthorized, ApiKeyError::Invalid)),
            };
        }

        return Outcome::Error((Status::InternalServerError, ApiKeyError::Invalid));
    }
}



#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let path = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("config.json");
    let file: std::fs::File = OpenOptions::new().read(true).write(true).create(true).open(path).expect("Failed to open config file.");
    file.lock_exclusive().expect("Could not lock the config file, is it open in another program?");

    let state = AppState::new(&file);

    let _ = fs2::FileExt::unlock(&file);

    let r = rocket::build()
        .configure(rocket::Config::figment().merge(("port", state.port)))
        .manage(state)
        .mount("/", routes![test])
        .attach(AdHoc::on_liftoff("Save Loop", |r| {
            Box::pin(async move {
                tokio::spawn(async move {
                    backend::save_loop().await;
                });
            })
        }))
        .ignite().await?;

    

    r.launch().await?;


    Ok(())
}
