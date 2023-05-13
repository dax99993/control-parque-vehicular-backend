use actix_web::{HttpResponse, web};
use anyhow::Context;
use common::models::vehicule::vehicule::FilterQueryVehicule;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e404};

use common::models::vehicule::{Vehiculo, EstadoVehiculo, VehiculoFiltrado};

use crate::routes::users::sqlx::obtener_usuario_por_id_sqlx;



#[tracing::instrument(
    name = "Get vehicule by id",
    skip(pool, session)
)]
pub async fn get_vehicule(
    session: JwtSession,
    pool: web::Data<PgPool>,
    uuid: web::Path<Uuid>,
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

    // Vehiculo valido ?
    let vehiculo = obtener_vehiculo_por_id_sqlx(&pool, &uuid).await
        .map_err(|_| e500())?
        .ok_or(e404().with_message("No se encontro el Vehiculo"))?;


    // Respuesta exitosa

    if usuario.es_admin() {
        let api_response = ApiResponse::<Vehiculo>::new()
            .with_message("Vehiculo")
            .with_data(vehiculo)
            .to_resp();
        
        return Ok(api_response);
    } else {
        let vehiculo_filtrado: VehiculoFiltrado = 
        if vehiculo.estado == EstadoVehiculo::Disponible && vehiculo.activo {
            VehiculoFiltrado::from(vehiculo)
        } else {
            // Not too sure if it should be a 404
            return Err(e404().with_message("No se encontro vehiculo"))?;
        };
        let api_response = ApiResponse::<VehiculoFiltrado>::new()
            .with_message("Vehiculo")
            .with_data(vehiculo_filtrado)
            .to_resp();
        
        return Ok(api_response);
    }
}

#[tracing::instrument(
    name = "Query vehiculo por id",
    skip(pool)
)]
pub async fn obtener_vehiculo_por_id_sqlx(
    pool: &PgPool,
    uuid: &Uuid,
) -> Result<Option<Vehiculo>, anyhow::Error> {
    let vehicule: Option<Vehiculo> = sqlx::query_as!(
        Vehiculo,
        r#"
        SELECT 
            vehiculo_id, marca, modelo, año,
            numero_placa,
            nombre_economico,
            numero_tarjeta,
            estado as "estado!: EstadoVehiculo",
            activo,
            imagen,
            creado_en,
            modificado_en
        FROM vehiculos
        WHERE vehiculo_id = $1
        "#,
        uuid
    )
    .fetch_optional(pool)
    .await
    .context("Fallo la ejecucion del query")?;

    Ok(vehicule)
}



#[tracing::instrument(
    name = "Get todos los vehiculos",
    skip(pool, session)
)]
pub async fn get_all_vehicules(
    session: JwtSession,
    pool: web::Data<PgPool>,
    query: web::Query<FilterQueryVehicule>
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

    // Validar query
    let query = query.into_inner();

    // Query vehiculo DB
    let vehiculos = obtener_vehiculos_con_filtro_sqlx(&pool, query).await
        .map_err(|_| e500())?;

    if usuario.es_admin() {
        let api_response = ApiResponse::<Vec<Vehiculo>>::new()
            .with_message("Lista de vehiculos")
            .with_data(vehiculos)
            .to_resp();
        
        return Ok(api_response);
    } else {
        let vehiculos_filtrados: Vec<VehiculoFiltrado> = 
            vehiculos
            .into_iter()
            .filter(|v| v.estado == EstadoVehiculo::Disponible && v.activo)
            .map(|v| VehiculoFiltrado::from(v))
            .collect();

        let api_response = ApiResponse::<Vec<VehiculoFiltrado>>::new()
            .with_message("Lista de vehiculos")
            .with_data(vehiculos_filtrados)
            .to_resp();
        
        return Ok(api_response);
    }
}


#[tracing::instrument(
    name = "Query vehiculos con filtro",
    skip(pool)
)]
pub async fn obtener_vehiculos_con_filtro_sqlx(
    pool: &PgPool,
    //query: &VehiculesQuery,
    filtro: FilterQueryVehicule,
) -> Result<Vec<Vehiculo>, anyhow::Error> {

    let mut query = sqlx::QueryBuilder::new(
        r#"SELECT *
        FROM (
            SELECT 
                vehiculo_id, marca, modelo, año,
                numero_placa,
                nombre_economico,
                numero_tarjeta,
                estado as "estado!: EstadoVehiculo",
                activo,
                imagen,
                creado_en,
                modificado_en
            FROM vehiculos
            ORDER BY creado_en DESC
        ) x
        WHERE creado_en <= now()"#);

    if filtro.marca.is_some() {
       query.push(" AND marca = ");
       query.push_bind(filtro.marca.unwrap());
    }
    if filtro.modelo.is_some() {
       query.push(" AND modelo = ");
       query.push_bind(filtro.modelo.unwrap());
    }
    if filtro.año.is_some() {
       query.push(" AND año = ");
       query.push_bind(filtro.año.unwrap());
    }
    if filtro.numero_placa.is_some() {
       query.push(" AND numero_placa = ");
       query.push_bind(filtro.numero_placa.unwrap());
    }
    if filtro.estado.is_some() {
       query.push(" AND estado = ");
       query.push_bind(filtro.estado.unwrap());
    }
    if filtro.activo.is_some() {
       query.push(" AND activo = ");
       query.push_bind(filtro.activo.unwrap());
    }

    // add page and limiter
    let pagina: i64 = filtro.pagina.unwrap_or(1).max(1);
    let vehiculos_por_pagina: i64 = filtro.limite.unwrap_or(3).min(5);
    query.push(" LIMIT ");
    query.push_bind( vehiculos_por_pagina);
    query.push(" OFFSET ");
    query.push_bind((pagina - 1) * vehiculos_por_pagina );

    tracing::info!("sql = {}", query.sql());
    let rows = query.build().fetch_all(pool).await.context("Fallo el query")?;

    dbg!("successful vehicules query");

    let vehiculos = rows.iter().map(|r| {
        dbg!("getting columns {}",&r.columns());
        Vehiculo {
            vehiculo_id: r.get("vehiculo_id"),
            marca: r.get("marca"),
            modelo: r.get("modelo"),
            año: r.get("año"),
            numero_placa: r.get("numero_placa"),
            nombre_economico: r.get("nombre_economico"),
            numero_tarjeta: r.get("numero_tarjeta"),
            estado: r.get("estado!: EstadoVehiculo"),
            activo: r.get("activo"),
            imagen: r.get("imagen"),
            modificado_en: r.get("modificado_en"),
            creado_en: r.get("creado_en"),
        }
    }).collect();


    Ok(vehiculos)
}
