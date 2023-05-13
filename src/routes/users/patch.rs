use actix_web::{HttpResponse, web, HttpRequest};
//use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use common::models::user::{Usuario, ActualizaUsuario};

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e403, e404};
//use crate::telemetry::spawn_blocking_with_tracing;

use super::sqlx::{obtener_usuario_por_id_sqlx, actualizar_usuario_sqlx, actualizar_imagen_usuario_sqlx};


#[tracing::instrument(
    name = "Actualizar Usuario por id",
    skip(session, pool)
)]
pub async fn user_patch(
    session: JwtSession,
    pool: web::Data<PgPool>,
    uuid: web::Path<Uuid>,
    body: web::Json<ActualizaUsuario>
) -> Result<HttpResponse, actix_web::Error> {

    // Usuario es admin ?
    let usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?
        .ok_or(e500())?;

    if !usuario.es_admin() {
        return Err(e403().with_message("No tienes los permisos requeridos"))?;
    }

    // Otro Usuario valido?
    let mut otro_usuario = obtener_usuario_por_id_sqlx(&pool, &uuid).await
        .map_err(|_| e500())?
        .ok_or(e404().with_message("No se encontro el usuario"))?;

    if otro_usuario.es_admin() && usuario.usuario_id != otro_usuario.usuario_id {
       return Err(e403().with_message("No puedes modificar otros administrador"))?; 
    }

    // Actualizar Usuario 
    let update_body = body.into_inner();
    // Deberia validar actualizacion
    // update_body.validate();
    otro_usuario.actualizar(update_body);

    // Query Actualizar DB
    let usuario_actualizado = actualizar_usuario_sqlx(&pool, otro_usuario).await
        .map_err(|_| e500())?;

    // Respuesta exitosa
    let api_response = ApiResponse::<Usuario>::new()
        .with_message("Usuario Actualizado")
        .with_data(usuario_actualizado)
        .to_resp();
    
    Ok(api_response)
}


use actix_multipart::Multipart;
use crate::upload::image::handle_picture_multipart;

#[tracing::instrument(
    name = "Actualizar imagen de Usuario por id",
    skip(session, pool, payload, req)
)]
pub async fn user_picture_patch(
    session: JwtSession,
    pool: web::Data<PgPool>,
    uuid: web::Path<Uuid>,
    payload: Multipart,
    req: HttpRequest, 
) -> Result<HttpResponse, actix_web::Error> {

    // Usuario es admin ?
    let usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?
        .ok_or(e500())?;

    if !usuario.es_admin() {
        return Err(e403().with_message("No tienes los privilegios necesarios"))?;
    }

    // Otro Usuario valido?
    let mut otro_usuario = obtener_usuario_por_id_sqlx(&pool, &uuid).await
        .map_err(|_| e500())?
        .ok_or(e404().with_message("No se encontro Usuario"))?;

    if otro_usuario.es_admin() && usuario.usuario_id != otro_usuario.usuario_id {
       return Err(e403().with_message("No puedes modificar otro administrador"))?; 
    }

    let picture_path = format!("./uploads/users/{}.jpeg", otro_usuario.usuario_id);
    
    // Guardar imagen
    handle_picture_multipart(payload, req, &picture_path, Some((1024,1024))).await
        .map_err(|_| e500())?;

    // Actualizar Usuario 
    otro_usuario.imagen = picture_path;

    let usuario_actualizado = actualizar_imagen_usuario_sqlx(&pool, otro_usuario).await
        .map_err(|_| e500())?;

    // Respuesta exitosa
    let api_response = ApiResponse::<Usuario>::new()
        .with_message("Usuario Actualizado")
        .with_data(usuario_actualizado)
        .to_resp();

    Ok(api_response)
}
