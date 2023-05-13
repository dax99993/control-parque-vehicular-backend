use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use uuid::Uuid;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e403, e404};

use crate::routes::users::sqlx::obtener_usuario_por_id_sqlx;


#[tracing::instrument(
    name = "Query borrar vehiculo",
    skip(pool)
)]
async fn borrar_vehiculo_sqlx(
    pool: &PgPool,
    uuid: &Uuid,
) -> Result<bool, sqlx::Error> {
    let query = sqlx::query!(
        r#"
        DELETE FROM vehiculos
        WHERE vehiculo_id = $1
        "#,
        uuid
    )
    .execute(pool)
    .await?;

    //dbg!("{}", &query);

    Ok(query.rows_affected() != 0)
}

#[tracing::instrument(
    name = "Borrar vehiculo por id",
    skip(pool, session)
)]
pub async fn delete_vehicule(
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

    // Query borrar vehiculo DB
    match borrar_vehiculo_sqlx(&pool, &uuid).await {
        Ok(deleted) => {
            if !deleted {
               return Err(e404().with_message("No se encontro el Vehiculo"))?;
            }
        },
        Err(_) => { 
            return Err(e500())?;
        },
    }

    // Respuesta exitosa
    let api_response = ApiResponse::<()>::new()
        .with_message("Vehiculo borrado")
        .to_resp();

    Ok(api_response)
}
