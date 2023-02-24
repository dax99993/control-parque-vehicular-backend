use actix_web::{HttpResponse, web, HttpRequest,  http::header::CONTENT_LENGTH };
use sqlx::PgPool;
use uuid::Uuid;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500};
use crate::models::user::{User, FilteredUser, UpdateUserMe};

use crate::routes::users::utils::{get_user_by_id_sqlx, update_user_sqlx, update_user_picture_sqlx};


#[tracing::instrument(
    name = "Patch me user",
    skip_all
)]
pub async fn user_patch_me(
    session: JwtSession,
    pool: web::Data<PgPool>,
    body: web::Json<UpdateUserMe>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = get_user_by_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    let user = user.ok_or(e500())?;


    let update_body = body.into_inner();
    let user = user.update_me(update_body);
    let updated_user = update_user_sqlx(&pool, user).await
        .map_err(|_| e500())?;

    let filter_user = FilteredUser::from(updated_user);
    let api_response = ApiResponse::<FilteredUser>::new()
        .with_message("Your new user info")
        .with_data(filter_user)
        .to_resp();
    Ok(api_response)
}

use actix_multipart::Multipart;
use crate::upload::image::handle_picture_multipart;

#[tracing::instrument(
    name = "Patch me user picture",
    skip_all
)]
pub async fn user_picture_patch_me(
    session: JwtSession,
    pool: web::Data<PgPool>,
    payload: Multipart,
    req: HttpRequest, 
) -> Result<HttpResponse, actix_web::Error> {
    let user = get_user_by_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    let mut user = user.ok_or(e500())?;

    let picture_path = format!("./uploads/users/{}.jpeg", user.user_id);
    
    handle_picture_multipart(payload, req, &picture_path).await
        .map_err(|_| e500())?;
    user.picture = picture_path;
    let updated_user = update_user_picture_sqlx(&pool, user).await
        .map_err(|_| e500())?;

    let filter_user = FilteredUser::from(updated_user);
    let api_response = ApiResponse::<FilteredUser>::new()
        .with_message("Updated User info")
        .with_data(filter_user)
        .to_resp();
    Ok(api_response)
}




