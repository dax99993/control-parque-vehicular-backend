use actix_web::{HttpResponse, web};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e403};
use crate::models::vehicule::{Vehicule, NewVehicule};

use crate::routes::user::utils::get_user_by_id;

//TODO update fn to insert picture
//and handle picture upload in http request
#[tracing::instrument(
    name = "Insert new vehicules query",
    skip(pool)
)]
async fn insert_new_vehicule_sqlx(
    pool: &PgPool,
    vehicule: NewVehicule,
) -> Result<Vehicule, anyhow::Error> {
    let vehicule: Vehicule = sqlx::query_as!(
        Vehicule,
        r#"
        INSERT INTO vehicules
        (vehicule_id, branch, model, year, number_plate, short_name, number_card)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
        Uuid::new_v4(),
        vehicule.branch,
        vehicule.model,
        vehicule.year,
        vehicule.number_plate,
        vehicule.short_name,
        vehicule.number_card,
    )
    .fetch_one(pool)
    .await
    .context("Failed to execute query")?;

    Ok(vehicule)
}

#[tracing::instrument(
    name = "Post new vehicules",
    skip(pool, session)
)]
pub async fn post_new_vehicule(
    session: JwtSession,
    pool: web::Data<PgPool>,
    vehicule: web::Json<NewVehicule>
) -> Result<HttpResponse, actix_web::Error> {
    let user = get_user_by_id(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    if user.is_none() {
       return Err(e500())?; 
    }
    
    if !user.unwrap().is_admin() {
        return Err(e403().with_message("You dont have required privilege"))?;
    }

    let new_vehicule = insert_new_vehicule_sqlx(&pool, vehicule.into_inner()).await
        .map_err(|_| e500())?;

    let api_response = ApiResponse::<Vehicule>::new()
        .with_message("New vehicule")
        .with_data(new_vehicule)
        .to_resp();

    Ok(api_response)
}
