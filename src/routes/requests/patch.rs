use actix_web::{HttpResponse, web, HttpRequest};
use anyhow::Context;
use common::models::vehicule::{Vehiculo, EstadoVehiculo, ActualizaVehiculo};
use sqlx::PgPool;
use uuid::Uuid;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e400, e403, e404};

use crate::routes::users::sqlx::obtener_usuario_por_id_sqlx;
use super::get::obtener_vehiculo_por_id_sqlx;

use crate::upload::image::get_uploads_path;


#[tracing::instrument(
    name = "Patch vehiculo",
    skip_all
)]
pub async fn patch_vehicule(
    session: JwtSession,
    pool: web::Data<PgPool>,
    uuid: web::Path<Uuid>,
    body: web::Json<ActualizaVehiculo>,
) -> Result<HttpResponse, actix_web::Error> {

    // Usuario es admin ?
    let usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?
        .ok_or(e500())?;

    if !usuario.es_admin() {
        return Err(e403().with_message("No tienes los permisos requeridos"))?;
    }

    // Query vehiculo DB
    let mut vehiculo = obtener_vehiculo_por_id_sqlx(&pool, &uuid).await
        .map_err(|_| e500())?
        .ok_or(e404().with_message("No se encontro el Vehiculo"))?;

    // Actualizar vehiculo
    vehiculo.actualizar(body.into_inner());

    // Query vehiculo actualizado DB
    let vehiculo_actualizado = actualizar_vehiculo_sqlx(&pool, vehiculo).await
        .map_err(|_| e500())?;

    //Respuesta exitosa
    let api_response = ApiResponse::<Vehiculo>::new()
        .with_message("Vehiculo Actualizado")
        .with_data(vehiculo_actualizado)
        .to_resp();

    Ok(api_response)
}

#[tracing::instrument(
    name = "Query Update vehiculo",
    skip(pool)
)]
async fn actualizar_vehiculo_sqlx(
    pool: &PgPool,
    vehiculo: Vehiculo,
) -> Result<Vehiculo, anyhow::Error> {

    let vehiculo_actualizado: Vehiculo = sqlx::query_as!(
        Vehiculo,
        r#"
        UPDATE vehiculos
        SET
            marca = $2, modelo = $3, año = $4,
            numero_placa = $5, nombre_economico = $6, numero_tarjeta = $7,
            estado = $8, activo = $9, imagen = $10,
            modificado_en = now()
        WHERE vehiculo_id = $1
        RETURNING 
            vehiculo_id, marca, modelo, año,
            numero_placa,
            nombre_economico,
            numero_tarjeta,
            estado as "estado!: EstadoVehiculo",
            activo,
            imagen,
            creado_en,
            modificado_en
        "#,
        vehiculo.vehiculo_id,
        vehiculo.marca,
        vehiculo.modelo,
        vehiculo.año,
        vehiculo.numero_placa,
        vehiculo.nombre_economico,
        vehiculo.numero_tarjeta,
        vehiculo.estado as EstadoVehiculo,
        vehiculo.activo,
        vehiculo.imagen,
    )
    .fetch_one(pool)
    .await
    .context("Fallo la ejecucion del query")?;

    Ok(vehiculo_actualizado)
}


use actix_multipart::Multipart;
use crate::upload::image::handle_picture_multipart;

#[tracing::instrument(
    name = "Actualizar imagen del vehiculo",
    skip(session, pool, payload, req)
)]
pub async fn patch_vehicule_picture(
    session: JwtSession,
    pool: web::Data<PgPool>,
    uuid: web::Path<Uuid>,
    payload: Multipart,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {

    // Usuario es admin ?
    let usuario = obtener_usuario_por_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?
        .ok_or(e500())?;

    if !usuario.es_admin() {
        return Err(e403().with_message("No tienes los permisos requeridos"))?;
    }


    // Query vehiculo DB
    let mut vehiculo = obtener_vehiculo_por_id_sqlx(&pool, &uuid).await
        .map_err(|_| e500())?
        .ok_or(e404().with_message("No se encontro el Vehiculo"))?;

    // Save image
    let base_path = get_uploads_path()
        .map_err(|_| e500())?
        .join("vehicules");

    let picture_filename = format!("{}-{}.jpeg", vehiculo.vehiculo_id, Uuid::new_v4().to_string());

    let save_path = base_path.join(&picture_filename);

    handle_picture_multipart(payload, req, &save_path.to_string_lossy(), None).await
        .map_err(|_| e500())?;


    // Actualizar vehiculo
    vehiculo.imagen = picture_filename;


    // Query vehiculo actualizado DB
    let vehiculo_actualizado = actualizar_vehiculo_sqlx(&pool, vehiculo).await
        .map_err(|_| e500())?;


    // Respuesta exitosa
    let api_response = ApiResponse::<Vehiculo>::new()
        .with_message("Vehiculo Actualizado")
        .with_data(vehiculo_actualizado)
        .to_resp();

    Ok(api_response)
}
