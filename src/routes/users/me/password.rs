use crate::api_response::{e400, e500, ApiResponse};

use crate::authentication::password::compute_password_hash;
use crate::authentication::jwt_session::JwtSession;

use crate::telemetry::spawn_blocking_with_tracing;

use crate::routes::users::utils::get_user_by_id_sqlx;

use actix_web::{HttpResponse, web};

use common::models::user::password::ChangePasswordMe;

use argon2::{Argon2, PasswordHash, PasswordVerifier};

use anyhow::Context;
use secrecy::{Secret, ExposeSecret};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;



#[tracing::instrument(
    name = "Change user password",
    skip_all,
    fields(user_id=tracing::field::Empty)
)]
pub async fn change_user_password(
    session: JwtSession,
    pool: web::Data<PgPool>,
    body: web::Json<ChangePasswordMe>,
) -> Result<HttpResponse, actix_web::Error> {
    // Check user validity
    let user = get_user_by_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    let user = user.ok_or(e500())?;

    // destructed body
    let password_form = body.0;

    if let Err(_) = password_form.validate() {
        return Err(e400().with_message("invalid body data"))?;
    }

    // Compare calculated hash from stored one
    let current_password = password_form.current_password.clone();
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| e500())?;
    Argon2::default().verify_password(&current_password.as_bytes(), &parsed_hash)
        .map_err(|_| e400().with_message("incorrect current password"))?;

    // Calculate new password hash from given password body
    let new_password = Secret::new(password_form.new_password.clone());
    let new_password_hash = spawn_blocking_with_tracing(
            move || compute_password_hash(new_password)
        )
        .await
        .map_err(|_| e500())?
        .map_err(|_| e500())?;


    // Store new password_hash
    insert_user_password_hash(&pool, new_password_hash, &user.user_id).await
        .map_err(|_| e500())?;

    //TODO add current token to blacklist to enforce relogin

    //Return sucessful response
    Ok(
        ApiResponse::<()>::new()
            .with_status_code(200)
            .with_message("User Password Changed")
            .to_resp()
    )


}


#[tracing::instrument(
    name = "insert new user password_hash",
    skip(pool, password_hash)
)]
async fn insert_user_password_hash(
    //transaction: &mut Transaction<'_, Postgres>,
    pool: &PgPool,
    password_hash: Secret<String>,
    user_id: &Uuid,
) -> Result<(), anyhow::Error> {

    let _ = sqlx::query!(
        r#"
        UPDATE users
        SET
        password_hash = $2,
        updated_at = now()
        WHERE user_id = $1
        "#,
        user_id,
        password_hash.expose_secret(),
    )
    .execute(pool)
    .await
    .context("Failed to update stored new user password_hash")?;
    
    Ok(())
}
