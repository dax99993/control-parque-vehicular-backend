use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use super::user::{User, UserStatus, Department};
use crate::state::AppState;


#[get("/")]
async fn get_users(app_state: AppState) -> HttpResponse {
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
        .fetch_all(&app_state.pool)
        .await
    {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}

#[get("/{user_id}")]
async fn get_user_by_id(path: web::Path<usize>, app_state: AppState) -> HttpResponse {
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

pub fn init(cfg: &mut web::ServiceConfig) {
    /*
    cfg.service(
        web::scope("/users")
        );
    */
    cfg.service(get_users);
    cfg.service(get_user_by_id);
}
