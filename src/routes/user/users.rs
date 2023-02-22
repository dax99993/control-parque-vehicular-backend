use actix_web::{HttpResponse, web};
use anyhow::Context;
use sqlx::PgPool;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e403, e400};
use crate::models::user::{User, FilteredUser};
//use crate::telemetry::spawn_blocking_with_tracing;

use super::utils::{get_users, get_user_by_id, delete_user_by_id};

#[tracing::instrument(
    name = "Get all users",
    skip_all
)]
pub async fn users_get_all(
    jwt_session: JwtSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    
    let user = get_user_by_id(&pool, &jwt_session.user_id).await
        .map_err(|_| e500())?;
    match user {
        Some(user) => {
            if user.is_admin() {
                let users = get_users(&pool).await
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
    skip(jwt_session, pool)
)]
pub async fn users_get_user_by_id(
    jwt_session: JwtSession,
    pool: web::Data<PgPool>,
    uuid: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    
    let user = get_user_by_id(&pool, &jwt_session.user_id).await
        .map_err(|_| e500())?;
    match user {
        Some(user) => {
            if user.is_admin() {
                let user = get_user_by_id(&pool, &uuid).await
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


#[tracing::instrument(
    name = "Delete user",
    skip(jwt_session, pool)
)]
pub async fn users_delete_user_by_id(
    jwt_session: JwtSession,
    pool: web::Data<PgPool>,
    uuid: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let uuid = uuid.into_inner();
    
    let user = get_user_by_id(&pool, &jwt_session.user_id).await
        .map_err(|_| e500())?;
    match user {
        Some(user) => {
            if user.is_admin() {
                let other_user = get_user_by_id(&pool, &uuid).await
                    .map_err(|_| e500())?;
                if other_user.is_none() {
                    return Err(e400().with_message("Non existing user"))?;
                }
                let other_user = other_user.unwrap();
                if other_user.is_admin() {
                    return Err(e403().with_message("Cannot delete admin user"))?;
                }

                delete_user_by_id(&pool, &uuid).await
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

use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use std::io::Write;
use uuid::Uuid;



#[tracing::instrument(
    name = "Save file",
    skip(file_data)
)]
async fn save_file(filepath: String, file_data: Vec<u8>) -> Result<(), anyhow::Error> {
    // File::create is blocking operation, use threadpool
    //let mut f = web::block(|| std::fs::File::create(filepath)).await??;
    let mut f = web::block(|| std::fs::File::create(filepath)).await
        .context("Couldn't create file with given filepath")?
        .context("Couldn't create blocking operation")?;
    // filesystem operations are blocking, we have to use threadpool
    web::block(move || f.write_all(&file_data)).await
        .context("Couldn't write file with given file_data")?
        .context("Couldn't create blocking operation")?;

    Ok(())
}

#[tracing::instrument(
    name = "Patch using Multipart",
    skip(payload)
)]
pub async fn user_patch(
    mut payload: Multipart,
) -> Result<HttpResponse, actix_web::Error> {
    // iterate over multipart stream
    let mut picture_filepath = String::from("");
    let mut picture: Vec<u8> = vec![];
    let mut body: Vec<u8> = vec![];
    
    while let Some(mut field) = payload.try_next().await? {
        // A multipart/form-data stream has to contain `content_disposition`
        let content_disposition = field.content_disposition();
        
        match field.name() {
            "picture" => {
                // inspect field content_type and check it contains image
                let filename = content_disposition
                    .get_filename()
                    .map(|n| format!("{}-{}",Uuid::new_v4().to_string(), n))
                    .unwrap();
                //.map_or_else(|| Uuid::new_v4().to_string(), sanitize_filename::sanitize);
                picture_filepath= format!("./tmp/{filename}");

                // Field in turn is stream of *Bytes* object
                while let Some(chunk) = field.try_next().await? {
                    picture.extend_from_slice(&chunk);
                }
            },
            "body" => {
                dbg!(&field);
                while let Some(chunk) = field.try_next().await? {
                    body.extend_from_slice(&chunk);
                }
            },
            _ => {
                dbg!(field);
            },
        }

    }
    let json: FilteredUser = serde_json::from_slice(&body).unwrap();
    dbg!(&json);

    if !picture.is_empty() {
        save_file(picture_filepath.clone(), picture).await
            .map_err(|_| e500())?;
    }

    Ok( HttpResponse::Ok().finish() )
}
