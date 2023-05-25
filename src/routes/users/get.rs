use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use uuid::Uuid;

use common::models::user::Usuario;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e403, e404};

use super::sqlx::{obtener_usuarios_sqlx, obtener_usuario_por_id_sqlx};


#[tracing::instrument(
    name = "Obtener todos los usuarios",
    skip_all
)]
pub async fn users_get_all(
    session: JwtSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {

    // Usuario es admin ?
    let usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?
        .ok_or(e500())?;

    if !usuario.es_admin() {
        return Err(e403().with_message("No tienes los permisos requeridos"))?;
    }

    // Obtener Usuario de DB
    let usuarios = obtener_usuarios_sqlx(&pool).await
        .map_err(|_| e500())?;

    // Respuesta exitosa
    let api_response = ApiResponse::<Vec<Usuario>>::new()
        .with_message("Lista de Usuarios")
        .with_data(usuarios)
        .to_resp();

    Ok(api_response)
}


#[tracing::instrument(
    name = "Obtener usuario",
    skip(session, pool)
)]
pub async fn users_get_user_by_id(
    session: JwtSession,
    pool: web::Data<PgPool>,
    uuid: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    
    // Usuario es admin ?
    let usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?
        .ok_or(e500())?;

    if !usuario.es_admin() {
        return Err(e403().with_message("No tienes los permisos requeridos"))?;
    }

    // Otro Usuario valido?
    let otro_usuario = obtener_usuario_por_id_sqlx(&pool, &uuid).await
        .map_err(|_| e500())?;
    let otro_usuario = otro_usuario.ok_or(e404().with_message("No se encontro el Usuario"))?;


    // Respuesta exitosa
    let api_response = ApiResponse::<Usuario>::new()
        .with_message("Usuario")
        .with_data(otro_usuario)
        .to_resp();

    Ok(api_response)
}
