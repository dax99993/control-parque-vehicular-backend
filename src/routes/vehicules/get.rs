use actix_web::{HttpResponse, web};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e404};
use crate::models::vehicule::{Vehicule, FilteredVehicule};

use crate::routes::user::utils::get_user_by_id;


#[tracing::instrument(
    name = "Query vehicule",
    skip(pool)
)]
async fn get_vehicule_sqlx(
    pool: &PgPool,
    uuid: &Uuid,
) -> Result<Option<Vehicule>, anyhow::Error> {
    let vehicule: Option<Vehicule> = sqlx::query_as!(
        Vehicule,
        r#"
        SELECT *
        FROM vehicules
        WHERE vehicule_id = $1
        "#,
        uuid
    )
    .fetch_optional(pool)
    .await
    .context("Failed to execute query")?;

    Ok(vehicule)
}

#[tracing::instrument(
    name = "Get vehicule by id",
    skip(pool, session)
)]
pub async fn get_vehicule(
    session: JwtSession,
    pool: web::Data<PgPool>,
    uuid: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    // Get user making the request
    let user = get_user_by_id(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    let user = user.ok_or(e500())?;

    let vehicule = get_vehicule_sqlx(&pool, &uuid).await
        .map_err(|_| e500())?;
    let vehicule = vehicule.ok_or(e404().with_message("Vehicule not found"))?;

    if user.is_admin() {
        let api_response = ApiResponse::<Vehicule>::new()
            .with_message("Vehicule")
            .with_data(vehicule)
            .to_resp();
        
        return Ok(api_response);
    } else {
        let filtered_vehicule: FilteredVehicule = 
        if vehicule.is_available() && vehicule.is_active() {
            FilteredVehicule::from(vehicule)
        } else {
            return Err(e404().with_message("Vehicule not found"))?;
        };
        let api_response = ApiResponse::<FilteredVehicule>::new()
            .with_message("Vehicule")
            .with_data(filtered_vehicule)
            .to_resp();
        
        return Ok(api_response);
    }
}

#[tracing::instrument(
    name = "Query all vehicules",
    skip(pool)
)]
async fn get_all_vehicules_sqlx(
    pool: &PgPool,
) -> Result<Vec<Vehicule>, anyhow::Error> {
    let vehicules: Vec<Vehicule> = sqlx::query_as!(
        Vehicule,
        r#"
        SELECT *
        FROM vehicules
        "#
    )
    .fetch_all(pool)
    .await
    .context("Failed to execute query")?;

    Ok(vehicules)
}

#[tracing::instrument(
    name = "Get all vehicules",
    skip(pool, session)
)]
pub async fn get_all_vehicules(
    session: JwtSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = get_user_by_id(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    let user = user.ok_or(e500())?;

    let vehicules = get_all_vehicules_sqlx(&pool).await
        .map_err(|_| e500())?;

    if user.is_admin() {
        let api_response = ApiResponse::<Vec<Vehicule>>::new()
            .with_message("List of vehicules")
            .with_data(vehicules)
            .to_resp();
        
        return Ok(api_response);
    } else {
        let filtered_vehicules: Vec<FilteredVehicule> = 
            vehicules
            .into_iter()
            .filter(|veh| veh.is_available() && veh.is_active())
            .map(|v| FilteredVehicule::from(v))
            .collect();
        let api_response = ApiResponse::<Vec<FilteredVehicule>>::new()
            .with_message("List of vehicules")
            .with_data(filtered_vehicules)
            .to_resp();
        
        return Ok(api_response);
    }
}
