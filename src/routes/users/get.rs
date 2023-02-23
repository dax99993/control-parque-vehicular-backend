use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use uuid::Uuid;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e403, e400};
use crate::models::user::User;
//use crate::telemetry::spawn_blocking_with_tracing;

use super::utils::{get_users_sqlx, get_user_by_id_sqlx};


#[tracing::instrument(
    name = "Get all users",
    skip_all
)]
pub async fn users_get_all(
    session: JwtSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {

    let user = get_user_by_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    match user {
        Some(user) => {
            if user.is_admin() {
                let users = get_users_sqlx(&pool).await
                    .map_err(|_| e500())?;
                let api_response = ApiResponse::<Vec<User>>::new()
                    .with_message("List of Users")
                    .with_data(users)
                    .to_resp();
                Ok(api_response)
            } else {
                return Err(e403().with_message("You dont have required privilege"))?;
            }
        },
        None => {
            return Err(e500())?;
        }
    }
}

#[tracing::instrument(
    name = "Get user",
    skip(session, pool)
)]
pub async fn users_get_user_by_id(
    session: JwtSession,
    pool: web::Data<PgPool>,
    uuid: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    
    let user = get_user_by_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    match user {
        Some(user) => {
            if user.is_admin() {
                let user = get_user_by_id_sqlx(&pool, &uuid).await
                    .map_err(|_| e500())?;
                if user.is_none() {
                    return Err(e400().with_message("Non existing user"))?;
                }

                let api_response = ApiResponse::<User>::new()
                    .with_message("Query Users")
                    .with_data(user.unwrap())
                    .to_resp();
                Ok(api_response)
            } else {
                return Err(e403().with_message("You dont have required privilege"))?;
            }
        },
        None => {
            return Err(e500())?;
        }
    }
}
