use crate::models::user::RegisterUser;
use crate::authentication::password::compute_password_hash;
use crate::telemetry::spawn_blocking_with_tracing;
use crate::error::error_chain_fmt;
use actix_web::{HttpResponse, web};
use anyhow::Context;
use sqlx::PgPool;
use secrecy::{Secret, ExposeSecret};
use uuid::Uuid;
use validator::Validate;

struct NewUser {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password_hash: Secret<String>,
}


#[derive(thiserror::Error)]
pub enum RegisterError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for RegisterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}



#[tracing::instrument(
    name = "Register user",
    skip(body, pool)
)]
pub async fn register_user(
    pool: web::Data<PgPool>,
    body: web::Json<RegisterUser>,
) -> Result<HttpResponse, actix_web::Error> {

    /* Validate body register */
    let register_user = body.into_inner();
    if register_user.validate().is_err() {
        return Ok(HttpResponse::BadRequest().json(
                serde_json::json!({"status": "fail", "message": "Registration body invalid"})
                ));
    }

    /* check passwords match */
    if register_user.password.expose_secret() !=
       register_user.re_password.expose_secret() {
        return Ok(HttpResponse::BadRequest().json(
                serde_json::json!({"status": "fail", "message": "Password don't match"})
                ));
    }

    if register_user.password.expose_secret().len() < 6 ||
       register_user.password.expose_secret().len() > 255 {
        return Ok(HttpResponse::BadRequest().json(
                serde_json::json!({"status": "fail", "message": "Password should be between 6 and 255 characters"})
                ));
    }
    
    /* verify if user with given email exists, in case it does conflict */
    match exists_user_with_email(&pool, &register_user.email)
        .await
        .context("Failed to query existing user.") {
        Ok(exists) => {
            if exists {
                return Ok(HttpResponse::Conflict().json(
                        serde_json::json!({"status": "fail", "message": "Account with given email already exists"})
                        ));
            }
        }
        Err(_) => {
                return Ok(HttpResponse::InternalServerError().json(
                        serde_json::json!({"status": "fail", "message": "Server error"})
                        ));
            }
        }

    //register in database
    let user_id = match insert_user(register_user, &pool).await {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json(
                        serde_json::json!({"status": "fail", "message": "Server error"})
                        ));
        }
    };
    tracing::Span::current()
        .record("user_id", &tracing::field::display(&user_id));

    //TODO
    //generate registration token
    //send email to verify user

    Ok(HttpResponse::Created().json(
            serde_json::json!({"status": "sucess", "message": "User created"})
    ))
}


#[tracing::instrument(
    name = "Querying user existence",
    skip(email, pool)
)]
async fn exists_user_with_email(pool: &PgPool, email: &str) -> Result<bool, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT EXISTS(
            SELECT user_id FROM users
            WHERE email = $1
        )
        "#,
        email
        )
        .fetch_one(pool)
        .await?;

    let user_exists = row.exists.unwrap();

    Ok(user_exists)
}

#[tracing::instrument(name = "insert new user", skip(user, pool))]
async fn insert_user(
    user: RegisterUser,
    pool: &PgPool,
) -> Result<uuid::Uuid, anyhow::Error> {
    let password_hash = spawn_blocking_with_tracing(
        move || compute_password_hash(user.password)
        )
        .await?
        .context("Failed to hash password")?;

    /*
    let user = NewUser { 
        first_name: user.first_name,
        last_name: user.last_name,
        email: user.email,
        password_hash
    };
    */

    let uuid = Uuid::new_v4();
    let row = sqlx::query!(
        r#"
        INSERT INTO users
        (user_id, first_name, last_name, email, password_hash)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING user_id
        "#,
        uuid,
        user.first_name,
        user.last_name,
        user.email,
        password_hash.expose_secret(),
    )
    .fetch_one(pool)
    .await
    .context("Failed to performed a query to retrieve stored new user")?;
    //.map(|row| row.user_id);
    
    Ok(row.user_id)
}
