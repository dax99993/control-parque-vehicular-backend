use actix_web::{HttpResponse, web, HttpRequest};
use anyhow::Context;
use common::models::vehicule::{Vehiculo, EstadoVehiculo, ActualizaVehiculo};
use sqlx::PgPool;
use uuid::Uuid;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e400, e403, e404};

use crate::routes::users::utils::get_user_by_id_sqlx;
use super::get::get_vehiculo_sqlx;



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
    let user = get_user_by_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    let user = user.ok_or(e500())?;
    if !user.is_admin() {
        return Err(e403().with_message("No tienes los permisos requeridos"))?;
    }

    let vehiculo = get_vehiculo_sqlx(&pool, &uuid).await
        .map_err(|_| e500())?;
    let mut vehiculo = vehiculo.ok_or(e404().with_message("No se encontro el Vehiculo"))?;

    //let vehiculo = vehiculo.actualizar(body.into_inner());
    vehiculo.actualizar(body.into_inner());

    let vehiculo_actualizado = update_vehiculo_sqlx(&pool, vehiculo).await
        .map_err(|_| e500())?;

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
async fn update_vehiculo_sqlx(
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
    name = "Patch vehicule picture",
    skip(session, pool, payload, req)
)]
pub async fn patch_vehicule_picture(
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
        return Err(e403().with_message("No tienes los permisos requeridos"))?;
    }

    let vehiculo = get_vehiculo_sqlx(&pool, &uuid).await
        .map_err(|_| e500())?;
    let mut vehiculo = vehiculo.ok_or(e404().with_message("No se encontro el Vehiculo"))?;

    let base_path = "./uploads/";
    let picture_path = format!("vehicules/{}-{}.jpeg", vehiculo.vehiculo_id, Uuid::new_v4().to_string());
    let save_path = format!("{base_path}{picture_path}");

    handle_picture_multipart(payload, req, &save_path, None).await
        .map_err(|_| e500())?;
    vehiculo.imagen = picture_path;

    let vehiculo_actualizado = update_vehiculo_sqlx(&pool, vehiculo).await
        .map_err(|_| e500())?;

    let api_response = ApiResponse::<Vehiculo>::new()
        .with_message("Vehiculo Actualizado")
        .with_data(vehiculo_actualizado)
        .to_resp();

    Ok(api_response)
}
