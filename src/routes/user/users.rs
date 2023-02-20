use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e403};
use crate::models::user::User;

use super::utils::{get_users, get_user_by_id};


pub async fn user_get_all(
    jwt_session: JwtSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    
    let user = get_user_by_id(&pool, jwt_session.user_id).await
        .map_err(|_| e500())?;
    match user {
        Some(user) => {
            if user.is_admin() {
                let users = get_users(&pool).await
                    .map_err(|_| e500())?;
                let api_response = ApiResponse::<Vec<User>>::new()
                    .with_message("Users info")
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
