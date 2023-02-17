use actix_web::{HttpResponse, web, Responder};
use sqlx::PgPool;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, ApiError};
use crate::models::user::{User, FilteredUser};

use super::user::get_user_by_id;


pub async fn user_get_me(
    jwt_session: JwtSession,
    pool: web::Data<PgPool>,
) -> Result<impl Responder, actix_web::Error> {
    
    /*
    if let Ok(Some(user)) = get_user_by_id(&pool, jwt_session.user_id).await
        .map_err(|e| Err(ApiError::new()
            .with_status("fail")
            .with_message("Server Error")
            )
        )? {
    */
    let user = get_user_by_id(&pool, jwt_session.user_id).await
        .map_err(|_| ApiError::new()
            .with_status_code(500)
            .with_status("fail")
            .with_message("Server Error")
            )?;
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
            return Err(ApiError::new()
                       .with_status("fail")
                       .with_message("Server Error")
                      )?;
        }
    }
}
