use actix_web::{HttpResponse, web};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e403};

use crate::routes::users::utils::get_user_by_id_sqlx;

use common::models::vehicule::{NuevoVehiculo, Vehiculo, EstadoVehiculo};

//TODO update fn to insert picture
//and handle picture upload in http request
#[tracing::instrument(
    name = "Insert new vehicules query",
    skip(pool)
)]
async fn insert_nuevo_vehiculo_sqlx(
    pool: &PgPool,
    vehicule: NuevoVehiculo,
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
        vehicule.marca,
        vehicule.modelo,
        vehicule.año,
        vehicule.numero_placa,
        vehicule.nombre_economico,
        vehicule.numero_tarjeta,
    )
    .fetch_one(pool)
    .await
    .context("Fallo la ejecucion del query")?;

    Ok(vehiculo)
}

#[tracing::instrument(
    name = "Post nuevo vehiculo",
    skip(pool, session)
)]
pub async fn post_new_vehicule(
    session: JwtSession,
    pool: web::Data<PgPool>,
    vehiculo: web::Json<NuevoVehiculo>
) -> Result<HttpResponse, actix_web::Error> {
    let user = get_user_by_id_sqlx(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    let user = user.ok_or(e500())?;
    if !user.is_admin() {
        return Err(e403().with_message("No tienes los permisos requeridos"))?;
    }

    let nuevo_vehiculo = insert_nuevo_vehiculo_sqlx(&pool, vehiculo.into_inner()).await
        .map_err(|_| e500())?;

    let api_response = ApiResponse::<Vehiculo>::new()
        .with_message("Nuevo vehiculo")
        .with_data(nuevo_vehiculo)
        .to_resp();

    Ok(api_response)
}
