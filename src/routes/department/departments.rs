use actix_web::{HttpResponse, web};
use futures::TryFutureExt;
use sqlx::PgPool;
use anyhow::Context;

use crate::{authentication::jwt_session::JwtSession, models::department::Department, api_response::{e500, ApiResponse, e404}};


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
async fn get_department_with_id_sqlx(
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
    name = "Insert new department",
    skip_all
)]
async fn insert_department_with_name_sqlx(
    pool: &PgPool,
    name: String,
) -> Result<Department, anyhow::Error> {
    //let row = sqlx::query!(
    let department: Department = sqlx::query_as!(
        Department,
        r#"
        INSERT INTO departments
        (name)
        VALUES ($1)
        RETURNING *
        "#,
        name)
        .fetch_one(pool)
        .await
        .context("Failed to execute query")?;

    Ok(department)
}

#[tracing::instrument(
    name = "Delete department with id",
    skip_all
)]
async fn delete_department_with_name_sqlx(
    pool: &PgPool,
    id: i32,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        r#"
        DELETE FROM departments
        WHERE id = $1
        "#,
        id)
        .execute(pool)
        .await
        .context("Failed to execute query")?;

    if query.rows_affected() == 0 {
        return Err(anyhow::anyhow!("Non existing department"));
    }

    Ok(())
}

#[tracing::instrument(
    name = "Update department with id",
    skip_all
)]
async fn update_department_with_name_sqlx(
    pool: &PgPool,
    id: i32,
    name: String,
) -> Result<Department, anyhow::Error> {
    //let row = sqlx::query!(
    let department: Department = sqlx::query_as!(
        Department,
        r#"
        UPDATE departments
        SET name = $2
        WHERE id = $1
        RETURNING *
        "#,
        id,
        name)
        .fetch_one(pool)
        .await
        .context("Failed to execute query")?;

    /*
    if query.rows_affected() == 0 {
        return Err(anyhow::anyhow!("Could not update department"));
    }
    */

    Ok(department)
}

#[tracing::instrument(
    name = "Get departments",
    skip_all
)]
pub async fn departments_get(
    _jwt_session: JwtSession,
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
pub async fn department_get_with_id(
    _jwt_session: JwtSession,
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> Result<HttpResponse, actix_web::Error> {
    let department = get_department_with_id_sqlx(&pool, id.into_inner()).await
        .map_err(|_| e500())?;

    if department.is_none() {
       Err(e404().with_message("Deparment with given id doesn't exist"))?; 
    }

    let api_response = ApiResponse::<Department>::new()
        .with_message("Department")
        .with_data(department.unwrap())
        .to_resp();

    Ok(api_response)
}

#[tracing::instrument(
    name = "Create department",
    skip_all
)]
pub async fn department_post_with_name(
    _jwt_session: JwtSession,
    pool: web::Data<PgPool>,
    name: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let department = insert_department_with_name_sqlx(&pool, name.into_inner()).await
        .map_err(|_| e500())?;

    let api_response = ApiResponse::<Department>::new()
        .with_message("New department created")
        .with_data(department)
        .to_resp();

    Ok(api_response)
}


#[tracing::instrument(
    name = "Delete department",
    skip_all
)]
pub async fn department_delete_with_id(
    _jwt_session: JwtSession,
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> Result<HttpResponse, actix_web::Error> {
    // maybe match the error to return a bad request if non valid id
    delete_department_with_name_sqlx(&pool, id.into_inner()).await
        .map_err(|_| e500())?;

    let api_response = ApiResponse::<()>::new()
        .with_message("Department deleted")
        .to_resp();

    Ok(api_response)
}

#[derive(Debug, serde::Deserialize)]
pub struct DepartmentName {
    pub name: String
}


#[tracing::instrument(
    name = "Patch department",
    skip_all
)]
pub async fn department_patch(
    _jwt_session: JwtSession,
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
    department: web::Json<DepartmentName>,
) -> Result<HttpResponse, actix_web::Error> {
    // maybe match the error to return a bad request if non valid id
    let department = update_department_with_name_sqlx(&pool, id.into_inner(), department.into_inner().name).await
        .map_err(|_| e500())?;

    let api_response = ApiResponse::<Department>::new()
        .with_message("Department updated")
        .with_data(department)
        .to_resp();

    Ok(api_response)
}
