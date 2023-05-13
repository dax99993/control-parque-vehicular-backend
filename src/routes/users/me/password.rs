use crate::api_response::{e400, e500, ApiResponse};
use crate::authentication::password::compute_password_hash;
use crate::authentication::jwt_session::JwtSession;
use crate::telemetry::spawn_blocking_with_tracing;
use crate::routes::users::sqlx::obtener_usuario_por_id_sqlx;

use common::models::user::password::CambiarMiPassword;

use actix_web::{HttpResponse, web};
use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use secrecy::{Secret, ExposeSecret};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;



#[tracing::instrument(
    name = "Cambiar password de usuario",
    skip_all,
    fields(user_id=tracing::field::Empty)
)]
pub async fn change_user_password(
    session: JwtSession,
    pool: web::Data<PgPool>,
    body: web::Json<CambiarMiPassword>,
) -> Result<HttpResponse, actix_web::Error> {

    // Session actual tiene un usuario valido ?
    let usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?
        .ok_or(e500())?;


    // Destructed body and validate
    let password_form = body.0;

    if let Err(val_errors) = password_form.validate() {
        return Err(e400().with_message(format!("{:?}", val_errors)))?;
    }

    // Verificar password actual con password en DB
    let password_actual = password_form.password_actual.clone();

    let parsed_hash = PasswordHash::new(&usuario.password_hash)
        .map_err(|_| e500())?;

    Argon2::default().verify_password(&password_actual.as_bytes(), &parsed_hash)
        .map_err(|_| e400().with_message("Contraseña actual no concuerda"))?;

    // Calcular nuevo password hash con la nueva contraseña
    let nuevo_password = Secret::new(password_form.password_actual.clone());
    let nuevo_password_hash = spawn_blocking_with_tracing(
            move || compute_password_hash(nuevo_password)
        )
        .await
        .map_err(|_| e500())?
        .map_err(|_| e500())?;


    // Query insertar nueva contraseña
    insertar_usuario_password_hash(&pool, nuevo_password_hash, &usuario.usuario_id).await
        .map_err(|_| e500())?;


    //TODO add current token to blacklist to enforce relogin

    // Respuesta exitosa
    let api_response = ApiResponse::<()>::new()
            .with_status_code(200)
            .with_message("Se cambio la Contraseña del usuario")
            .to_resp();

    Ok(api_response)
}


#[tracing::instrument(
    name = "Insertar nuevo password_hash del usuario",
    skip(pool, password_hash)
)]
async fn insertar_usuario_password_hash(
    pool: &PgPool,
    password_hash: Secret<String>,
    usuario_id: &Uuid,
) -> Result<(), anyhow::Error> {

    let _ = sqlx::query!(
        r#"
        UPDATE usuarios
        SET
        password_hash = $2,
        modificado_en = now()
        WHERE usuario_id = $1
        "#,
        usuario_id,
        password_hash.expose_secret(),
    )
    .execute(pool)
    .await
    .context("Failed to update stored new user password_hash")?;
    
    Ok(())
}
