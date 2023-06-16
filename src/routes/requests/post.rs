use actix_web::{HttpResponse, web};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e403};

use crate::routes::users::sqlx::obtener_usuario_por_id_sqlx;

use common::models::request::{NuevaPeticion, Peticion};


#[tracing::instrument(
    name = "Query insertar nueva peticion",
    skip(pool)
)]
async fn insertar_nueva_peticion_sqlx(
    pool: &PgPool,
    //peticion: NuevaPeticion,
) -> Result<Vehiculo, anyhow::Error> {
    let vehiculo: Vehiculo = sqlx::query_as!(
        Vehiculo,
        r#"
        INSERT INTO vehiculos
        (vehiculo_id, marca, modelo, año, numero_placa, nombre_economico, numero_tarjeta)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
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
        Uuid::new_v4(),
        vehiculo.marca,
        vehiculo.modelo,
        vehiculo.año,
        vehiculo.numero_placa,
        vehiculo.nombre_economico,
        vehiculo.numero_tarjeta,
    )
    .fetch_one(pool)
    .await
    .context("Fallo la ejecucion del query")?;

    Ok(vehiculo)
}

#[tracing::instrument(
    name = "Post nueva peticion",
    skip(pool, session)
)]
pub async fn post_new_request(
    session: JwtSession,
    pool: web::Data<PgPool>,
    request: web::Json<NuevaPeticion>
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

    // Query insertar nuevo vehiculo DB
    let peticion = peticion.into_inner();
    let nueva_peticion = insertar_nueva_peticion_sqlx(&pool, peticion).await
        .map_err(|_| e500())?;

    // Respuesta exitosa
    let api_response = ApiResponse::<Vehiculo>::new()
        .with_message("Nuevo vehiculo")
        .with_data(nuevo_vehiculo)
        .to_resp();

    Ok(api_response)
}
