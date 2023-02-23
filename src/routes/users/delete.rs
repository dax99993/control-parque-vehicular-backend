use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use uuid::Uuid;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e403, e400};

use super::utils::{get_user_by_id_sqlx, delete_user_by_id_sqlx};


#[tracing::instrument(
    name = "Delete user",
    skip(session, pool)
)]
pub async fn users_delete_user_by_id(
    session: JwtSession,
    pool: web::Data<PgPool>,
    uuid: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = get_user_by_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    match user {
        Some(user) => {
            if user.is_admin() {
                let other_user = get_user_by_id_sqlx(&pool, &uuid).await
                    .map_err(|_| e500())?;
                if other_user.is_none() {
                    return Err(e400().with_message("Non existing user"))?;
                }
                let other_user = other_user.unwrap();
                if other_user.is_admin() {
                    return Err(e403().with_message("Cannot delete admin user"))?;
                }

                delete_user_by_id_sqlx(&pool, &uuid).await
                    .map_err(|_| e500())?;
                
                let api_response = ApiResponse::<()>::new()
                    .with_message("User deleted")
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
