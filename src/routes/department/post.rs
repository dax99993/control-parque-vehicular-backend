use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use anyhow::Context;

use crate::authentication::jwt_session::JwtSession;
use crate::models::department::Department;
use crate::api_response::{e500, ApiResponse, e403};

use crate::routes::users::utils::get_user_by_id_sqlx;


#[tracing::instrument(
    name = "Insert new department query",
    skip(pool)
)]
async fn insert_department_with_name_sqlx(
    pool: &PgPool,
    name: String,
) -> Result<Department, anyhow::Error> {
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
    name = "Create department",
    skip(session, pool)
)]
pub async fn department_post(
    session: JwtSession,
    pool: web::Data<PgPool>,
    name: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = get_user_by_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    if user.is_none() {
       return Err(e500())?; 
    }
    
    if !user.unwrap().is_admin() {
        return Err(e403().with_message("You dont have required privilege"))?;
    }

    let department = insert_department_with_name_sqlx(&pool, name.into_inner()).await
        .map_err(|_| e500())?;

    let api_response = ApiResponse::<Department>::new()
        .with_message("New department")
        .with_data(department)
        .to_resp();

    Ok(api_response)
}
