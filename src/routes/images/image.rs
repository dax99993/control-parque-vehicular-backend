use actix_web::{web, HttpResponse};
use actix_web::{HttpRequest, Responder};
use actix_files::NamedFile;

use crate::api_response::{e500, e404};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct QueryFile {
    pub filename: String
}

#[tracing::instrument(
    name = "Serve static image",
    skip(req)
)]
pub async fn get_image(
    req: HttpRequest,
    query: web::Query<QueryFile>,
) -> Result<HttpResponse, actix_web::Error> {
//) -> Result<impl Responder, actix_web::Error> {
    let base_path = std::env::current_dir().expect("Failed to determine current directory");
    let uploads_path = base_path.join("uploads");
    let file_path = uploads_path.join(&query.filename);
    //dbg!(&file_path);
    match NamedFile::open_async(file_path).await {
        Ok(f) =>  Ok(f.into_response(&req)),
        Err(e) => { 
            match e.kind() {
                std::io::ErrorKind::NotFound => { Err(e404().with_message("File not found"))? },
                _ => { Err(e500())? },

            }
        }
    }
}


#[tracing::instrument(
    name = "Post image",
    skip(req)
)]
pub async fn post_image(
    req: HttpRequest,
    query: web::Query<QueryFile>,
) -> Result<HttpResponse, actix_web::Error> {
//) -> Result<impl Responder, actix_web::Error> {
    let base_path = std::env::current_dir().expect("Failed to determine current directory");
    let uploads_path = base_path.join("uploads");
    let file_path = uploads_path.join(&query.filename);
    //dbg!(&file_path);
    match NamedFile::open_async(file_path).await {
        Ok(f) =>  Ok(f.into_response(&req)),
        Err(e) => { 
            match e.kind() {
                std::io::ErrorKind::NotFound => { Err(e404().with_message("File not found"))? },
                _ => { Err(e500())? },

            }
        }
    }
}

