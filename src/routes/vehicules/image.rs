use actix_web::{web, HttpResponse};
use actix_web::HttpRequest;
use actix_files::NamedFile;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{e500, e404};
use crate::upload::image::get_uploads_path;


#[tracing::instrument(
    name = "Serve imagen estatica del vehiculo",
    skip(req)
)]
pub async fn get_imagen_vehiculo(
    //_session: JwtSession,
    req: HttpRequest,
    file: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {

    // Obtener path
    let base_path = get_uploads_path()
        .map_err(|_| e500())?
        .join("vehicules");

    let file_path = base_path.join(&file.into_inner());
    //dbg!(&file_path);
    
    // Obtener el archivo y enviar respuesta
    match NamedFile::open_async(file_path).await {
        Ok(f) =>  Ok(f.into_response(&req)),
        Err(e) => { 
            match e.kind() {
                std::io::ErrorKind::NotFound => { Err(e404().with_message("No se encontro el archivo"))? },
                _ => { Err(e500())? },

            }
        }
    }
}
