use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use uuid::Uuid;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e403, e404};

use super::sqlx::{obtener_usuario_por_id_sqlx, borrar_usuario_por_id_sqlx};


#[tracing::instrument(
    name = "Borrar usuario",
    skip(session, pool)
)]
pub async fn users_delete_user_by_id(
    session: JwtSession,
    pool: web::Data<PgPool>,
    uuid: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {

    // Usuario es admin ?
    let usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        // Map SQLX error a 500
        .map_err(|_| e500())?
        // Si query regresa Some map a OK o error
        .ok_or(e500())?;

    if !usuario.es_admin() {
        return Err(e403().with_message("No tienes los permisos requeridos"))?;
    }

    // Otro Usuario valido ?
    let otro_usuario = obtener_usuario_por_id_sqlx(&pool, &uuid).await
        .map_err(|_| e500())?
        .ok_or(e404().with_message("No se encontro el usuario"))?;

    if otro_usuario.es_admin() && usuario.usuario_id != otro_usuario.usuario_id{
       return Err(e403().with_message("No puedes eliminar otro administrador!"))?; 
    }

    // Query borrar DB
    borrar_usuario_por_id_sqlx(&pool, &uuid).await
        .map_err(|_| e500())?;

    // Respuesta exitosa
    let api_response = ApiResponse::<()>::new()
        .with_message("Usuario eliminado")
        .to_resp();

    Ok(api_response)
}
