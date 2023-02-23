use actix_multipart::Multipart;
use actix_web::{HttpResponse, web, HttpRequest};
use anyhow::Context;
use sqlx::PgPool;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e400, e403, e404};
use crate::models::vehicule::{Vehicule, UpdateVehicule};

use crate::routes::user::utils::get_user_by_id;
use super::get::get_vehicule_sqlx;


use uuid::Uuid;
use tokio::io::AsyncWriteExt as _;
use image::{ DynamicImage, imageops::FilterType };
use actix_web::http::header::CONTENT_LENGTH;
//use futures::{StreamExt, TryStreamExt as _};
use futures::TryStreamExt as _;
use mime::{ Mime, IMAGE_PNG, IMAGE_JPEG, IMAGE_GIF, APPLICATION_JSON};
use crate::error::error_chain_fmt;

#[tracing::instrument(
    name = "Updated vehicule query",
    skip(pool)
)]
async fn update_vehicule_sqlx(
    pool: &PgPool,
    vehicule: Vehicule,
) -> Result<Vehicule, anyhow::Error> {
    let vehicule: Vehicule = sqlx::query_as!(
        Vehicule,
        r#"
        UPDATE vehicules
        SET
        branch = $2, model = $3, year = $4,
        number_plate = $5, short_name = $6, number_card = $7,
        status = $8, active = $9, picture = $10,
        updated_at = now()
        WHERE vehicule_id = $1
        RETURNING *
        "#,
        vehicule.vehicule_id,
        vehicule.branch,
        vehicule.model,
        vehicule.year,
        vehicule.number_plate,
        vehicule.short_name,
        vehicule.number_card,
        vehicule.status,
        vehicule.active,
        vehicule.picture,
    )
    .fetch_one(pool)
    .await
    .context("Failed to execute query")?;

    Ok(vehicule)
}

#[tracing::instrument(
    name = "Patch vehicule",
    skip_all
)]
pub async fn patch_vehicule(
    session: JwtSession,
    pool: web::Data<PgPool>,
    uuid: web::Path<Uuid>,
    payload: Multipart,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let user = get_user_by_id(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    if user.is_none() {
       return Err(e500())?; 
    }
    if !user.unwrap().is_admin() {
        return Err(e403().with_message("You dont have required privilege"))?;
    }

    let vehicule = get_vehicule_sqlx(&pool, &uuid).await
        .map_err(|_| e500())?;
    let vehicule = vehicule.ok_or(e404().with_message("Vehicule not found"))?;

    let update_vehicule = match handle_multipart(payload, req).await {
        Ok(update_vehicule) => {update_vehicule},
        Err(e) => {
            if e == HandleMultipartError::InvalidBodyError {
                return Err(e400().with_message("Invalid body"))?;
            } else {
                return Err(e500())?;
            }
        }
    };

    let vehicule = vehicule.update(update_vehicule);

    let updated_vehicule = update_vehicule_sqlx(&pool, vehicule).await
        .map_err(|_| e500())?;


    let api_response = ApiResponse::<Vehicule>::new()
        .with_message("Updated vehicule")
        .with_data(updated_vehicule)
        .to_resp();

    Ok(api_response)
}


#[tracing::instrument(
    name = "Handling multipart",
    skip(payload)
)]
async fn handle_multipart(
    mut payload: Multipart,
    req: HttpRequest,
) -> Result<UpdateVehicule, HandleMultipartError> {

    let content_length: usize = match req.headers().get(CONTENT_LENGTH) {
        Some(header_value) => header_value.to_str().unwrap_or("0").parse().unwrap(),
        None => "0".parse().unwrap(),
    } ;

    let max_file_size: usize = 1024 * 1024; //10 Mb
    let max_file_count: usize = 3;
    let legal_filetypes: [Mime; 4] = [IMAGE_GIF, IMAGE_PNG, IMAGE_JPEG, APPLICATION_JSON ];
    let mut current_count: usize = 0;
    let dir: &str = "./uploads/vehicules/";
    let mut picture_path = String::from("");
    let mut update_body: Vec<u8> = vec![];

    //dbg!(&content_length);
    if content_length > max_file_size { return Err(HandleMultipartError::FileTooBigError) };


    loop {
        if current_count == max_file_count { break; }
        if let Ok(Some(mut field)) = payload.try_next().await {
            //dbg!(&field);
            let filetype: Option<&Mime> = field.content_type();
            //dbg!(&filetype);
            if filetype.is_none() { continue; }
            if !legal_filetypes.contains(&filetype.unwrap()) { continue; }
            
            match field.name() {
                "body" => {
                    while let Ok(Some(chunk)) = field.try_next().await {
                        update_body.extend_from_slice(&chunk);
                    }
                },
                "picture" => {
                    let destination: String = format!(
                        "{}{}-{}",
                        dir,
                        Uuid::new_v4(),
                        field.content_disposition().get_filename().unwrap()
                    );
                    //dbg!(&destination);

                    //let mut saved_file = tokio::fs::File::create(&destination).await.unwrap();
                    let mut saved_file = match tokio::fs::File::create(&destination).await {
                        Ok(f) => {f},
                        Err(_) => {
                            return Err(HandleMultipartError::CreateFileError);
                        }
                    };
                    //dbg!(&saved_file);

                    while let Ok(Some(chunk)) = field.try_next().await {
                        //let _ = saved_file.write_all(&chunk).await.unwrap();
                        match saved_file.write_all(&chunk).await {
                            Ok(_) => {},
                            Err(_) => {
                                return Err(HandleMultipartError::SaveFileError);
                            },
                        }
                    }

                   picture_path = match 
                       web::block(move || async move {
                           write_image(&destination, &dir).await
                       }).await {
                        Ok(res) => {
                            match res.await {
                                Ok(path) => {path},
                                Err(e) => {
                                    return Err(e);
                                },
                            }
                        },
                        Err(_) => {
                            return Err(HandleMultipartError::SaveImageError);
                        },
                    };
                    dbg!(&picture_path);
                },
                _ => { continue; }
            }
        } else { break; }
        current_count += 1;
    }
    dbg!(&current_count);
    let mut update_vehicule: UpdateVehicule =
    if !update_body.is_empty() {
        //let update_vehicule = serde_json::from_slice(&update_body).unwrap();
        match serde_json::from_slice(&update_body) {
            Ok(json) => {json},
            Err(_) => {
                return Err(HandleMultipartError::InvalidBodyError);
            },
        }
    } else {
        return Err(HandleMultipartError::InvalidBodyError);
    };

    if picture_path != "".to_string() {
        update_vehicule.picture = Some(picture_path);
    }

    Ok( update_vehicule )
}

async fn write_image(destination: &str, dir: &str) -> Result<String, HandleMultipartError> {
    let uploaded_img: DynamicImage = match image::open(&destination) {
        Ok(image) => image,
        Err(_) => {
            return Err(HandleMultipartError::OpenImageError);
        },
    };
    match tokio::fs::remove_file(&destination).await {
        Ok(_) => {},
        Err(_) => {
            return Err(HandleMultipartError::DeleteFileError);
        },
    }
    let save_path = format!("{}{}.jpeg", dir, Uuid::new_v4().to_string());
    match uploaded_img
        .resize_exact(1024, 1024, FilterType::Nearest)
        .save(&save_path) {
            Ok(_) => {},
            Err(_) => {
                return Err(HandleMultipartError::SaveImageError);
            }
        }
    Ok(save_path)
}


#[derive(thiserror::Error, PartialEq, Eq)]
enum HandleMultipartError {
    #[error("File too big for uploading")]
    FileTooBigError,
    #[error("Invalid Body")]
    InvalidBodyError,
    #[error("Cannot Create File")]
    CreateFileError,
    #[error("Cannot Delete File")]
    DeleteFileError,
    #[error("Cannot Save File")]
    SaveFileError,
    #[error("Cannot Open Temporal Image")]
    OpenImageError,
    #[error("Cannot Save Permanent Image")]
    SaveImageError,
}

impl std::fmt::Debug for HandleMultipartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
