use actix_web::{HttpResponse, web};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e403};

use crate::routes::users::sqlx::obtener_usuario_por_id_sqlx;

use common::models::request::{NuevaPeticion, Peticion, EstadoPeticion};


#[tracing::instrument(
    name = "Query insertar nueva peticion",
    skip(pool)
)]
async fn insertar_nueva_peticion_sqlx(
    pool: &PgPool,
    peticion: NuevaPeticion,
    usuario_id: &Uuid,
    vehiculo_id: &Uuid,
) -> Result<Peticion, anyhow::Error> {
    let peticion: Peticion = sqlx::query_as!(
        Peticion,
        r#"
        INSERT INTO peticiones
        (peticion_id, usuario_id, vehiculo_id, inicio, finalizo, kilometraje_inicial, kilometraje_final, usuario_licencia_imagen)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING 
            peticion_id, usuario_id, vehiculo_id,
            inicio, finalizo,
            actividad_descripcion, actividad_comentario,
            kilometraje_inicial, kilometraje_final,
            estado as "estado!: EstadoPeticion",
            usuario_licencia_imagen,
            vehiculo_imagen,
            gasolina_imagen,
            creado_en,
            modificado_en
        "#,
        Uuid::new_v4(),
        usuario_id,
        vehiculo_id,
        peticion.inicio,
        peticion.finalizo,
        peticion.kilometraje_inicial,
        200000,
        //peticion.usuario_licencia_imagen,
        String::from("Image name"),
    )
    .fetch_one(pool)
    .await
    .context("Fallo la ejecucion del query")?;

    Ok(peticion)
}

//#[derive(Debug, serde::Deserialize)]
//pub struct VehiculoId(Uuid);

#[tracing::instrument(
    name = "Post nueva peticion",
    skip(pool, session)
)]
pub async fn post_new_request(
    session: JwtSession,
    pool: web::Data<PgPool>,
    //vehiculo_id: web::Path<VehiculoId>,
    vehiculo_id: web::Path<Uuid>,
    peticion: web::Json<NuevaPeticion>
) -> Result<HttpResponse, actix_web::Error> {

    // Usuario es admin ?
    let usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?
        .ok_or(e500())?;

    /*
    if !usuario.es_admin() {
        return Err(e403().with_message("No tienes los permisos requeridos"))?;
    }
    */

    let vehiculo_id = vehiculo_id.into_inner();

    // Query insertar nuevo vehiculo DB
    let peticion = peticion.into_inner();
    let nueva_peticion = insertar_nueva_peticion_sqlx(&pool, peticion, &session.user_id, &vehiculo_id).await
        .map_err(|_| e500())?;

    // Respuesta exitosa
    let api_response = ApiResponse::<Peticion>::new()
        .with_message("Nuevo peticion")
        .with_data(nueva_peticion)
        .to_resp();

    Ok(api_response)
}
