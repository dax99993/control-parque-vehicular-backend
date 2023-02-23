use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use anyhow::Context;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{e500, ApiResponse, e403, e404};

//use super::get::department_get_with_id;
use crate::routes::users::utils::get_user_by_id_sqlx;


#[tracing::instrument(
    name = "Delete department query",
    skip_all
)]
async fn delete_department_with_name_sqlx(
    pool: &PgPool,
    id: i32,
) -> Result<bool, anyhow::Error> {
    let query = sqlx::query!(
        r#"
        DELETE FROM departments
        WHERE id = $1
        "#,
        id)
        .execute(pool)
        .await
        .context("Failed to execute query")?;


    Ok(query.rows_affected() != 0)
}


#[tracing::instrument(
    name = "Delete department",
    skip_all
)]
pub async fn delete_department(
    session: JwtSession,
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = get_user_by_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    if user.is_none() {
       return Err(e500())?; 
    }
    
    if !user.unwrap().is_admin() {
        return Err(e403().with_message("You dont have required privilege"))?;
    }
    
    match delete_department_with_name_sqlx(&pool, id.into_inner()).await {
        Ok(deleted) => {
            if !deleted {
               return Err(e404().with_message("Department not found"))?;
            }
        },
        Err(_) => { 
            return Err(e500())?;
        },
    }

    let api_response = ApiResponse::<()>::new()
        .with_message("Department deleted")
        .to_resp();

    Ok(api_response)
}
