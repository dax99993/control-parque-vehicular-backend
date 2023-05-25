use actix_web::{web, HttpResponse};
use actix_web::HttpRequest;
use actix_files::NamedFile;

use sqlx::PgPool;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{e500, e404, e403};

use crate::upload::image::get_uploads_path;
use crate::routes::users::sqlx::obtener_usuario_por_id_sqlx;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct File {
    pub filename: String
}

#[tracing::instrument(
    name = "Serve imagen estatica del usuario",
    skip(session, pool, req)
)]
pub async fn get_imagen_usuario(
    session: JwtSession,
    pool: web::Data<PgPool>,
    req: HttpRequest,
    file: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    // Usuario es admin ?
    let usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?
        .ok_or(e500())?;

    if !usuario.es_admin() {
        return Err(e403().with_message("No tienes los permisos requeridos"))?;
    }

    // Obtener path
    let base_path = get_uploads_path()
        .map_err(|_| e500())?
        .join("users");

    let file = file.into_inner();
    let file_path = base_path.join(&file);
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
