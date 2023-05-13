use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use anyhow::Context;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{e500, ApiResponse, e403, e404};

//use super::get::department_get_with_id;
use crate::routes::users::sqlx::obtener_usuario_por_id_sqlx;


#[tracing::instrument(
    name = "Query borrar departamento",
    skip_all
)]
async fn borrar_departamento_por_nombre_sqlx(
    pool: &PgPool,
    id: i32,
) -> Result<bool, anyhow::Error> {

    let query = sqlx::query!(
        r#"
        DELETE FROM departamentos
        WHERE id = $1
        "#,
        id)
        .execute(pool)
        .await
        .context("Failed to execute query")?;


    Ok(query.rows_affected() != 0)
}


#[tracing::instrument(
    name = "Borrar departamento por id",
    skip_all
)]
pub async fn delete_department(
    session: JwtSession,
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> Result<HttpResponse, actix_web::Error> {

    // Usuario es admin ?
    let usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?
        .ok_or(e500())?;
    
    if !usuario.es_admin() {
        return Err(e403().with_message("No tienes los permisos requeridos"))?;
    }
    
    
    // Query Borrar DB
    match borrar_departamento_por_nombre_sqlx(&pool, id.into_inner()).await {
        Ok(deleted) => {
            if !deleted {
               return Err(e404().with_message("No se encontro el departamento"))?;
            }
        },
        Err(_) => { 
            return Err(e500())?;
        },
    }

    // Respuesta exitosa
    let api_response = ApiResponse::<()>::new()
        .with_message("Departament eliminado")
        .to_resp();

    Ok(api_response)
}
