use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500};
//use crate::models::user::FilteredUser;
use crate::models::user::User;

use crate::routes::users::utils::get_user_by_id_sqlx;


#[tracing::instrument(
    name = "User get me",
    skip_all,
)]
pub async fn user_get_me(
    session: JwtSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = get_user_by_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    let user = user.ok_or(e500())?;

    //let filter_user = FilteredUser::from(user);
    //let api_response = ApiResponse::<FilteredUser>::new()
    let api_response = ApiResponse::<User>::new()
        .with_message("Your user info")
        //.with_data(filter_user)
        .with_data(user)
        .to_resp();
    Ok(api_response)
}
