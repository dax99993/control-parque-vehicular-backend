use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use anyhow::Context;

use crate::authentication::jwt_session::JwtSession;
use crate::models::department::Department;
use crate::api_response::{e500, ApiResponse, e404};


#[tracing::instrument(
    name = "Query all departments",
    skip_all
)]
async fn get_departments_sqlx(
    pool: &PgPool
) -> Result<Vec<Department>, anyhow::Error> {
    let departments: Vec<Department> = sqlx::query_as!(
        Department,
        r#"
        SELECT *
        FROM departments
        "#)
        .fetch_all(pool)
        .await
        .context("Failed to execute query")?;

    Ok(departments)
}

#[tracing::instrument(
    name = "Query department with id",
    skip_all
)]
pub async fn get_department_with_id_sqlx(
    pool: &PgPool,
    id: i32,
) -> Result<Option<Department>, anyhow::Error> {
    let departments: Option<Department> = sqlx::query_as!(
        Department,
        r#"
        SELECT *
        FROM departments
        WHERE id = $1
        "#,
        id)
        .fetch_optional(pool)
        .await
        .context("Failed to execute query")?;

    Ok(departments)
}

#[tracing::instrument(
    name = "Get departments",
    skip_all
)]
pub async fn departments_get(
    _session: JwtSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {

    let departments = get_departments_sqlx(&pool).await
        .map_err(|_| e500())?;

    let api_response = ApiResponse::<Vec<Department>>::new()
        .with_message("List of departments")
        .with_data(departments)
        .to_resp();

    Ok(api_response)
}


#[tracing::instrument(
    name = "Get department with id",
    skip_all
)]
pub async fn department_get(
    _session: JwtSession,
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> Result<HttpResponse, actix_web::Error> {

    let department = get_department_with_id_sqlx(&pool, id.into_inner()).await
        .map_err(|_| e500())?;

    if department.is_none() {
       Err(e404().with_message("Department not found"))?; 
    }

    let api_response = ApiResponse::<Department>::new()
        .with_message("Department")
        .with_data(department.unwrap())
        .to_resp();

    Ok(api_response)
}
