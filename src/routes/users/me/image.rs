use actix_web::{web, HttpResponse};
use actix_web::HttpRequest;
use actix_files::NamedFile;

use sqlx::PgPool;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{e500, e404, e403};

use crate::upload::image::get_uploads_path;
use crate::routes::users::sqlx::obtener_usuario_por_id_sqlx;


#[tracing::instrument(
    name = "Serve imagen estatica de mi perfil usuario",
    skip(session, pool, req)
)]
pub async fn get_imagen_usuario(
    session: JwtSession,
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    // Usuario es valido?
    let usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?
        .ok_or(e500())?;

    // Obtener path
    let base_path = get_uploads_path()
        .map_err(|_| e500())?
        .join("users");

    let file_path = base_path.join(&usuario.imagen);
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
