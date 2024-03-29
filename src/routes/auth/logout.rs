use actix_web::HttpResponse;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500};


#[tracing::instrument(
    name = "Logout user",
    skip_all,
)]
pub async fn logout_user(
    session: JwtSession,
) -> Result<HttpResponse, actix_web::Error> {
    match session.blacklist_session() {
        Ok(_) => {
            Ok(ApiResponse::<()>::new().with_message("You have logout").to_resp())
        },
        Err(_) => { 
            Err(e500())?
        },
    }

}
