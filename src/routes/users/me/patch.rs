use actix_web::{HttpResponse, web, HttpRequest,  http::header::CONTENT_LENGTH };
use sqlx::PgPool;
use uuid::Uuid;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500};
use crate::models::user::{User, FilteredUser};

use crate::routes::users::utils::get_user_by_id_sqlx;



use actix_multipart::Multipart;
use futures::TryStreamExt as _;
use mime::{ Mime, IMAGE_PNG, IMAGE_JPEG, IMAGE_GIF, APPLICATION_JSON };

pub async fn user_patch_me(
    jwt_session: JwtSession,
    pool: web::Data<PgPool>,
    mut payload: Multipart,
    req: HttpRequest, 
) -> Result<HttpResponse, actix_web::Error> {
    let user = get_user_by_id_sqlx(&pool, &jwt_session.user_id).await
        .map_err(|_| e500())?;
    if user.is_none() {
        return Err(e500())?;
    }

    let user = user.unwrap();
    if user.is_admin() {
        let api_response = ApiResponse::<User>::new()
            .with_message("Your user info")
            .with_data(user)
            .to_resp();
        Ok(api_response)
    } else {
        let _user_multipart = handle_multipart(payload, req).await.unwrap();
        let filter_user = FilteredUser::from(user);
        /*
        if user_multipart.picture_name.is_some() {
            filter_user.picture = user_multipart.picture_name.unwrap();
        }
        if user_multipart.update_body.is_some() {
            let update_body:  = serde_json::from_str(&user_multipart.update_body.unwrap())
                .unwrap();
            
            filter_user.picture = user_multipart.picture_name.unwrap();
        }
        filter_user.pic
        */
        let api_response = ApiResponse::<FilteredUser>::new()
            .with_message("Your user info")
            .with_data(filter_user)
            .to_resp();
        Ok(api_response)
    }
}

pub struct UserMeMultipart {
    pub picture_name: Option<String>,
    pub update_body: Option<String>,
}

use tokio::io::AsyncWriteExt as _;
use image::{ DynamicImage, imageops::FilterType };

async fn handle_multipart(mut payload: Multipart, req: HttpRequest) -> Result<UserMeMultipart, anyhow::Error> {

    let content_length: usize = match req.headers().get(CONTENT_LENGTH) {
        Some(header_value) => header_value.to_str().unwrap_or("0").parse().unwrap(),
        None => "0".parse().unwrap(),
    } ;

    let max_file_size: usize = 10_000;
    let max_file_count: usize = 2;
    let legal_filetypes: [Mime; 4] = [IMAGE_GIF, IMAGE_PNG, IMAGE_JPEG, APPLICATION_JSON];
    let mut current_count: usize = 0;
    let dir: &str = "./uploads/users/";
    let mut picture_path = String::from("");
    let mut update_body: Vec<u8> = vec![];

    if content_length > max_file_size { return Err(anyhow::anyhow!("Badrequest")) };


    loop {
        if current_count == max_file_count { break; }
        if let Ok(Some(mut field)) = payload.try_next().await {
            let filetype: Option<&Mime> = field.content_type();
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

                    let mut saved_file = tokio::fs::File::create(&destination).await.unwrap();
                    while let Ok(Some(chunk)) = field.try_next().await {
                        let _ = saved_file.write_all(&chunk).await.unwrap();
                    }


                   picture_path = web::block(move || async move {
                        let uploaded_img: DynamicImage = image::open(&destination).unwrap();
                        let _ = tokio::fs::remove_file(&destination).await.unwrap();
                        let new_picture_path = format!("{}{}.jpeg", dir, Uuid::new_v4().to_string());
                        uploaded_img
                            .resize_exact(1024, 1024, FilterType::Nearest)
                            .save(&picture_path).unwrap();
                        new_picture_path
                    }).await.unwrap().await;
                },
                _ => { continue; }
            }
        } else { break; }
        current_count += 1;
    }
    Ok( UserMeMultipart { picture_name: Some(picture_path.clone()), update_body: Some(String::from_utf8(update_body).unwrap()) } )
}
