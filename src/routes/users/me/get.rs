use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use common::models::user::Usuario;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500};
use crate::routes::users::sqlx::obtener_usuario_por_id_sqlx;


#[tracing::instrument(
    name = "Obtenener mi Usuario",
    skip_all,
)]
pub async fn user_get_me(
    session: JwtSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {

    // Sesion actual tiene un usuario valido ?
    let usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?
        .ok_or(e500())?;

    // Talvez deberia utilizar otro query con referencias joined

    // Respuesta exitosa 
    let api_response = ApiResponse::<Usuario>::new()
        .with_message("Tu informacion de usuario")
        .with_data(usuario)
        .to_resp();
    Ok(api_response)
}
