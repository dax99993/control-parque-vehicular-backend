use actix_web::{HttpResponse, web, HttpRequest};
use sqlx::PgPool;

use common::models::user::{Usuario, ActualizaMiUsuario};

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500};

use crate::routes::users::sqlx::{obtener_usuario_por_id_sqlx, actualizar_usuario_sqlx, actualizar_imagen_usuario_sqlx};


#[tracing::instrument(
    name = "Actualizar mi usuario",
    skip_all
)]
pub async fn user_patch_me(
    session: JwtSession,
    pool: web::Data<PgPool>,
    body: web::Json<ActualizaMiUsuario>,
) -> Result<HttpResponse, actix_web::Error> {

    // Sesion actual tiene un usuario valido ?
    let mut usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?
        .ok_or(e500())?;
    
    // Validar actualizacion
    let update_body = body.into_inner();
    //update_body.validate();

    // Actualizar usuario
    usuario.actualizar_me(update_body);

    // Query actualizar DB
    let usuario_actualizado = actualizar_usuario_sqlx(&pool, usuario).await
        .map_err(|_| e500())?;


    // Respuesta exitosa
    let api_response = ApiResponse::<Usuario>::new()
        .with_message("Usuario actualizado")
        .with_data(usuario_actualizado)
        .to_resp();

    Ok(api_response)
}

use actix_multipart::Multipart;
use crate::upload::image::handle_picture_multipart;

#[tracing::instrument(
    name = "Patch me user picture",
    skip_all
)]
pub async fn user_picture_patch_me(
    session: JwtSession,
    pool: web::Data<PgPool>,
    payload: Multipart,
    req: HttpRequest, 
) -> Result<HttpResponse, actix_web::Error> {

    // Sesion actual tiene un usuario valido ?
    let mut usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?
        .ok_or(e500())?;

    // Guardar Imagen
    let picture_path = format!("./uploads/users/{}.jpeg", usuario.usuario_id);

    handle_picture_multipart(payload, req, &picture_path, Some((1024,1024))).await
        .map_err(|_| e500())?;

    // Actualizar Usuario
    usuario.imagen= picture_path;

    // Query Actulizar Imagen DB
    let usuario_actualizado = actualizar_imagen_usuario_sqlx(&pool, usuario).await
        .map_err(|_| e500())?;

    // Respueta exitosa
    let api_response = ApiResponse::<Usuario>::new()
        .with_message("Usuario actualizado")
        .with_data(usuario_actualizado)
        .to_resp();
    Ok(api_response)
}

