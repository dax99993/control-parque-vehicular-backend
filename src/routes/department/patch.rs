use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use anyhow::Context;

use crate::authentication::jwt_session::JwtSession;
use crate::models::department::Department;
use crate::api_response::{e500, ApiResponse, e403, e404};

use crate::routes::users::utils::get_user_by_id_sqlx;
use super::get::get_department_with_id_sqlx;


#[tracing::instrument(
    name = "Update department query",
    skip_all
)]
async fn update_department_with_name_sqlx(
    pool: &PgPool,
    id: i32,
    name: String,
) -> Result<Department, anyhow::Error> {
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


    Ok(department)
}


#[tracing::instrument(
    name = "Patch department",
    skip_all
)]
pub async fn patch_department(
    session: JwtSession,
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
    body: web::Json<DepartmentName>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = id.into_inner();

    let user = get_user_by_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    if user.is_none() {
       return Err(e500())?; 
    }
    
    if !user.unwrap().is_admin() {
        return Err(e403().with_message("You dont have required privilege"))?;
    }
    
    let department = get_department_with_id_sqlx(&pool, id).await
        .map_err(|_| e500())?;
    department.ok_or(e404().with_message("Department not found"))?;

    let updated_department = update_department_with_name_sqlx(&pool, id, body.into_inner().name).await
        .map_err(|_| e500())?;

    let api_response = ApiResponse::<Department>::new()
        .with_message("Department updated")
        .with_data(updated_department)
        .to_resp();

    Ok(api_response)
}

#[derive(Debug, serde::Deserialize)]
pub struct DepartmentName {
    pub name: String
}
