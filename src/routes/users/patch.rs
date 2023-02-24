use actix_web::{HttpResponse, web, HttpRequest};
//use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e403, e404};
use crate::models::user::{User, UpdateUser};
//use crate::telemetry::spawn_blocking_with_tracing;

use super::utils::{get_user_by_id_sqlx, update_user_sqlx, update_user_picture_sqlx};


#[tracing::instrument(
    name = "Patch User",
    skip(session, pool)
)]
pub async fn user_patch(
    session: JwtSession,
    pool: web::Data<PgPool>,
    uuid: web::Path<Uuid>,
    update_body: web::Json<UpdateUser>
) -> Result<HttpResponse, actix_web::Error> {
    let user = get_user_by_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    let user = user.ok_or(e500())?;
    if !user.is_admin() {
        return Err(e403().with_message("You dont have required privilege"))?;
    }

    let other_user = get_user_by_id_sqlx(&pool, &uuid).await
        .map_err(|_| e500())?;
    let other_user = other_user.ok_or(e404().with_message("User not found"))?;
    if other_user.is_admin() && user.user_id != other_user.user_id {
       return Err(e403().with_message("Cannot patch other admin users"))?; 
    }

    // Get the patch update_body data
    // parse it into UpdateUser struct
    let update_body = update_body.into_inner();
    // Need to validate update_body since it can contain a email ****
    // update other_user
    let updated_user = other_user.update(update_body);
    // query to database
    let updated_user = update_user_sqlx(&pool, updated_user).await
        .map_err(|_| e500())?;
    // return updated user
    let api_response = ApiResponse::<User>::new()
        .with_message("Updated user")
        .with_data(updated_user)
        .to_resp();
    
    Ok(api_response)
}


use actix_multipart::Multipart;
use crate::upload::image::handle_picture_multipart;

#[tracing::instrument(
    name = "Patch user picture",
    skip(session, pool, payload, req)
)]
pub async fn user_picture_patch(
    session: JwtSession,
    pool: web::Data<PgPool>,
    uuid: web::Path<Uuid>,
    payload: Multipart,
    req: HttpRequest, 
) -> Result<HttpResponse, actix_web::Error> {
    let user = get_user_by_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    let user = user.ok_or(e500())?;
    if !user.is_admin() {
        return Err(e403().with_message("You dont have required privilege"))?;
    }

    let other_user = get_user_by_id_sqlx(&pool, &uuid).await
        .map_err(|_| e500())?;
    let mut other_user = other_user.ok_or(e404().with_message("User not found"))?;
    if other_user.is_admin() && user.user_id != other_user.user_id {
       return Err(e403().with_message("Cannot patch other admin users"))?; 
    }

    let picture_path = format!("./uploads/users/{}.jpeg", other_user.user_id);
    
    // Need to add a better error handling for this function
    // and add resize externally
    handle_picture_multipart(payload, req, &picture_path, Some((1024,1024))).await
        .map_err(|_| e500())?;
    other_user.picture = picture_path;
    let updated_user = update_user_picture_sqlx(&pool, other_user).await
        .map_err(|_| e500())?;

    let api_response = ApiResponse::<User>::new()
        .with_message("Updated User")
        .with_data(updated_user)
        .to_resp();
    Ok(api_response)
}
