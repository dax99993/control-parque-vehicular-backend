use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500};
use crate::models::user::{User, FilteredUser};

use super::user::get_user_by_id;


pub async fn user_get_me(
    jwt_session: JwtSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    
    let user = get_user_by_id(&pool, jwt_session.user_id).await
        .map_err(|_| e500())?;
    match user {
        Some(user) => {
            if user.is_admin() {
                let api_response = ApiResponse::<User>::new()
                    .with_message("Your user info")
                    .with_data(user)
                    .to_resp();
                Ok(api_response)
            } else {
                let filter_user = FilteredUser::from(user);
                let api_response = ApiResponse::<FilteredUser>::new()
                    .with_message("Your user info")
                    .with_data(filter_user)
                    .to_resp();
                Ok(api_response)
            }
        },
        None => {
            return Err(e500())?;
        }
    }
}
