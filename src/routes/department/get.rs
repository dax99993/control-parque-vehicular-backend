use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use anyhow::Context;

use crate::authentication::jwt_session::JwtSession;
//use crate::models::department::Department;
use common::models::department::Departamento;
use crate::api_response::{e500, ApiResponse, e404};

use crate::routes::users::sqlx::obtener_usuario_por_id_sqlx;


#[tracing::instrument(
    name = "Query all departments",
    skip_all
)]
async fn obtener_departamentos_sqlx(
    pool: &PgPool
) -> Result<Vec<Departamento>, anyhow::Error> {
    let departamentos: Vec<Departamento> = sqlx::query_as!(
        Departamento,
        r#"
        SELECT *
        FROM departamentos
        "#)
        .fetch_all(pool)
        .await
        .context("Failed to execute query")?;

    Ok(departamentos)
}

#[tracing::instrument(
    name = "Query department with id",
    skip_all
)]
pub async fn obtener_departamento_por_id_sqlx(
    pool: &PgPool,
    id: i32,
) -> Result<Option<Departamento>, anyhow::Error> {

    let departamento: Option<Departamento> = sqlx::query_as!(
        Departamento,
        r#"
        SELECT *
        FROM departamentos
        WHERE id = $1
        "#,
        id)
        .fetch_optional(pool)
        .await
        .context("Failed to execute query")?;

    Ok(departamento)
}

#[tracing::instrument(
    name = "Obtener departamentos",
    skip_all
)]
pub async fn departments_get(
    session: JwtSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {

    // Session actual tiene un usuario valido ?
    let _usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?
        .ok_or(e500())?;

    // Query departamentos DB
    let departamentos = obtener_departamentos_sqlx(&pool).await
        .map_err(|_| e500())?;

    // Respuesta exitosa
    let api_response = ApiResponse::<Vec<Departamento>>::new()
        .with_message("Lista de departamentos")
        .with_data(departamentos)
        .to_resp();

    Ok(api_response)
}


#[tracing::instrument(
    name = "Get department with id",
    skip_all
)]
pub async fn department_get(
    session: JwtSession,
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> Result<HttpResponse, actix_web::Error> {

    // Session actual tiene un usuario valido ?
    let _usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?
        .ok_or(e500())?;

    // Query departamento DB
    let departamento = obtener_departamento_por_id_sqlx(&pool, id.into_inner()).await
        .map_err(|_| e500())?
        .ok_or(e404().with_message("Department not found"))?;


    // Respuesta exitosa
    let api_response = ApiResponse::<Departamento>::new()
        .with_message("Departamento")
        .with_data(departamento)
        .to_resp();

    Ok(api_response)
}
