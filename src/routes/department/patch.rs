use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use anyhow::Context;

use common::models::department::{Departamento, ActualizaDepartamento};

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{e500, ApiResponse, e403, e404};
use crate::routes::users::sqlx::obtener_usuario_por_id_sqlx;
use super::get::obtener_departamento_por_id_sqlx;


#[tracing::instrument(
    name = "Query actualizar departamento",
    skip_all
)]
async fn actualizar_departamento_sqlx(
    pool: &PgPool,
    departamento: Departamento,
) -> Result<Departamento, anyhow::Error> {
    let departamento: Departamento = sqlx::query_as!(
        Departamento,
        r#"
        UPDATE departamentos
        SET nombre = $2
        WHERE id = $1
        RETURNING *
        "#,
        departamento.id,
        departamento.nombre)
        .fetch_one(pool)
        .await
        .context("Failed to execute query")?;


    Ok(departamento)
}


#[tracing::instrument(
    name = "Patch department",
    skip_all
)]
pub async fn patch_department(
    session: JwtSession,
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
    body: web::Json<ActualizaDepartamento>,
) -> Result<HttpResponse, actix_web::Error> {

    // Usuario es admin ?
    let usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?
        .ok_or(e500())?;
    
    if !usuario.es_admin() {
        return Err(e403().with_message("No tienes los permisos requeridos"))?;
    }
    

    // Departamento es valido ?
    let id = id.into_inner();
    let mut departamento = obtener_departamento_por_id_sqlx(&pool, id).await
        .map_err(|_| e500())?
        .ok_or(e404().with_message("No se encontro el departamento"))?;


    //Actualizar departamento
    departamento.nombre = body.into_inner().nombre;

    
    // Query actualizar departamento DB
    let departmento_actualizado = actualizar_departamento_sqlx(&pool, departamento).await
        .map_err(|_| e500())?;


    // Respuesta exitosa
    let api_response = ApiResponse::<Departamento>::new()
        .with_message("Departamento actualizado")
        .with_data(departmento_actualizado)
        .to_resp();

    Ok(api_response)
}
