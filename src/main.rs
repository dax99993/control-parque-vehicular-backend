#[macro_use]
extern crate nonblock_logger;

mod json_serialization;
mod models;
mod views;

pub mod state;
pub mod config;
pub mod database;


use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;
use crate::state::State;


struct AppStateWithCounter {
    counter: Mutex<i32>, // <- Mutex is necessary to mutate safely across threads
}


async fn index(data: web::Data<AppStateWithCounter>) -> String {
    let mut counter = data.counter.lock().unwrap(); // <- get counter's MutexGuard
    *counter += 1; // <- access counter inside MutexGuard

    format!("Request number: {counter}") // <- response with count
}


async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Logger
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    
    // Get Environmental variables
    let port = std::env::var("API_PORT").unwrap_or("8080".to_string());
    let address = format!("127.0.0.1:{port}");

    println!("Running Actix Web server at {}", address);

    // Get Appication State (Database pool)
    let app_state = State::init().await;

    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });
    
    HttpServer::new(move || {

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .app_data(counter.clone())
            .service(web::scope("/users").configure(models::user::routes::init))
            .route("/hey", web::get().to(manual_hello))
            .route("/", web::get().to(index))
    })
    .bind(address)
    .unwrap_or_else(|err| panic!("Couldn't start the server in port {} {:?}", port, err))
    .run()
    .await
}
