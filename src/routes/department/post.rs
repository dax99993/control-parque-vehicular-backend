use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use anyhow::Context;

use common::models::department::{Departamento, NuevoDepartamento};

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{e500, ApiResponse, e403};

use crate::routes::users::sqlx::obtener_usuario_por_id_sqlx;


#[tracing::instrument(
    name = "Query insertar nuevo departamento",
    skip(pool)
)]
async fn insertar_departmamento_con_nombre_sqlx(
    pool: &PgPool,
    nombre: String,
) -> Result<Departamento, anyhow::Error> {
    let departamento: Departamento = sqlx::query_as!(
        Departamento,
        r#"
        INSERT INTO departamentos
        (nombre)
        VALUES ($1)
        RETURNING *
        "#,
        nombre)
        .fetch_one(pool)
        .await
        .context("Failed to execute query")?;

    Ok(departamento)
}


#[tracing::instrument(
    name = "Crear departamento",
    skip(session, pool)
)]
pub async fn department_post(
    session: JwtSession,
    pool: web::Data<PgPool>,
    body: web::Json<NuevoDepartamento>,
) -> Result<HttpResponse, actix_web::Error> {

    // Usuario es admin ?
    let usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?
        .ok_or(e500())?;

    if !usuario.es_admin() {
        return Err(e403().with_message("No tienes los permisos requeridos"))?;
    }

    // Query insertar departamento DB
    let nombre = body.into_inner().nombre;
    let nuevo_departmento = insertar_departmamento_con_nombre_sqlx(&pool, nombre).await
        .map_err(|_| e500())?;

    // Respuesta exitosa
    let api_response = ApiResponse::<Departamento>::new()
        .with_message("Nuevo departamento")
        .with_data(nuevo_departmento)
        .to_resp();

    Ok(api_response)
}
