mod models;

use models::user::{User, UserStatus};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use sqlx::{
    postgres::{PgPoolOptions, PgRow},
    PgPool,
};
use std::sync::Mutex;

struct AppStateWithCounter {
    counter: Mutex<i32>, // <- Mutex is necessary to mutate safely across threads
}

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

async fn index(data: web::Data<AppStateWithCounter>) -> String {
    let mut counter = data.counter.lock().unwrap(); // <- get counter's MutexGuard
    *counter += 1; // <- access counter inside MutexGuard

    format!("Request number: {counter}") // <- response with count
}

#[get("/users")]
async fn get_users(app_state: web::Data<AppState>) -> HttpResponse {
    match sqlx::query_as!(
        User, 
        r#"SELECT 
        id,
        first_name,
        last_name,
        email,
        password,
        employee_number,
        active,
        picture,
        department,
        status as "status: UserStatus",
        created_at,
        updated_at
        FROM users"#
        )
        .fetch_one(&app_state.pool)
        .await
    {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}

#[get("/users/{user_id}")]
async fn get_user(path: web::Path<usize>, app_state: web::Data<AppState>) -> HttpResponse {
    let user_id: usize = path.into_inner();

    let user: Option<User> = sqlx::query_as!(
        User,
        r#"SELECT 
        id,
        first_name,
        last_name,
        email,
        password,
        employee_number,
        active,
        picture,
        department,
        status as "status: UserStatus",
        created_at,
        updated_at
        FROM users WHERE id = $1"#,
        user_id as i64,
    )
    .fetch_optional(&app_state.pool)
    .await
    .unwrap();

    match user {
        Some(user) => HttpResponse::Ok().json(user),
        None => HttpResponse::NotFound().json("No user found"),
    }
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
    let port = std::env::var("API_PORT").unwrap_or("8080".to_string());
    let address = format!("127.0.0.1:{port}");

    let pool: PgPool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    let app_state = AppState { pool };

    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });
    HttpServer::new(move || {
        println!("Running Actix Web server");

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .app_data(counter.clone())
            .service(get_user)
            .service(get_users)
            .route("/hey", web::get().to(manual_hello))
            .route("/", web::get().to(index))
    })
    .bind(address)
    .unwrap_or_else(|err| panic!("Couldn't start the server in port {} {:?}", port, err))
    .run()
    .await
}
