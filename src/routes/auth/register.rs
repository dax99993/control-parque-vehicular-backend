use crate::models::user::RegisterUser;
//use crate::authentication::password::compute_password_hash;
//use crate::telemetry::spawn_blocking_with_tracing;
use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use secrecy::{Secret, ExposeSecret};
use validator::{Validate, ValidationError};


#[tracing::instrument(name = "Register user", skip(body, pool))]
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
       register_user.password_verify.expose_secret() {
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
    match exists_user_with_email(&pool, &register_user.email).await {
        Ok(exists) => {
            if exists {
                return Ok(HttpResponse::Conflict().json(
                        serde_json::json!({"status": "fail", "message": "User already exists"})
                        ));
            }
        }
        Err(_) => {
                return Ok(HttpResponse::InternalServerError().json(
                        serde_json::json!({"status": "fail", "message": "Server error"})
                        ));
            }
        }

    //TODO
    //generate password hash
    //register in database
    //generate registration token
    //send email to verify user

    Ok(HttpResponse::Ok().json(
            serde_json::json!({"status": "sucess", "message": "User created"})
    ))
}


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
        .await
        .expect("Failed to execute query");

    let user_exists = row.exists.unwrap();

    Ok(user_exists)
}
