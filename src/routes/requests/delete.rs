use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use uuid::Uuid;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e403, e404};

use crate::routes::users::sqlx::obtener_usuario_por_id_sqlx;


//TODO agregar condicion para solo poder eliminar peticiones con estado que no se finalizado
#[tracing::instrument(
    name = "Query borrar peticion",
    skip(pool)
)]
async fn borrar_peticion_con_id_sqlx(
    pool: &PgPool,
    peticion_id: &Uuid,
    usuario_id: &Uuid,
) -> Result<bool, sqlx::Error> {
    let query = sqlx::query!(
        r#"
        DELETE FROM peticiones 
        WHERE
        peticion_id = $1 AND
        usuario_id = $2
        "#,
        peticion_id,
        usuario_id,
    )
    .execute(pool)
    .await?;

    Ok(query.rows_affected() != 0)
}

#[tracing::instrument(
    name = "Borrar peticion por id",
    skip(pool, session)
)]
pub async fn delete_request(
    session: JwtSession,
    pool: web::Data<PgPool>,
    uuid: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {

    // Usuario es admin ?
    let usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?
        .ok_or(e500())?;

    /*
    if !usuario.es_admin() {
        return Err(e403().with_message("No tienes los permisos requeridos"))?;
    }
    */

    // Query borrar vehiculo DB
    match borrar_peticion_con_id_sqlx(&pool, &uuid, &session.user_id).await {
        Ok(deleted) => {
            if !deleted {
               return Err(e404().with_message("No se encontro la peticion"))?;
            }
        },
        Err(_) => { 
            return Err(e500())?;
        },
    }

    // Respuesta exitosa
    let api_response = ApiResponse::<()>::new()
        .with_message("peticion borrada")
        .to_resp();

    Ok(api_response)
}
