mod user;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;

struct AppStateWithCounter {
    counter: Mutex<i32>, // <- Mutex is necessary to mutate safely across threads
}

async fn index(data: web::Data<AppStateWithCounter>) -> String {
    let mut counter = data.counter.lock().unwrap(); // <- get counter's MutexGuard
    *counter += 1; // <- access counter inside MutexGuard

    format!("Request number: {counter}") // <- response with count
}

#[get("/hello")]
async fn hello() -> impl Responder {
    let mut user = user::User::new("Manuel".to_string(), "Martinez".to_string());
    user.to_admin();
    HttpResponse::Ok().json(user)
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Get Environmental variables
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").unwrap();
    println!("Database url: {}", database_url);
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let address = format!("127.0.0.1:{port}");

    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });
    HttpServer::new(move || {
        println!("Running Actix Web server");

        App::new()
            .app_data(counter.clone())
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
            .route("/", web::get().to(index))
    })
    .bind(address)?
    .run()
    .await
}

