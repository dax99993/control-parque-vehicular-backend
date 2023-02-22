use actix_web::{HttpResponse, web};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e403, e400};
use crate::models::vehicule::{Vehicule, NewVehicule};

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
    let user = get_user_by_id(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    if user.is_none() {
       return Err(e500())?; 
    }
   
    let user = user.unwrap();

    let vehicule = get_vehicule_sqlx(&pool, &uuid).await
        .map_err(|_| e500())?;
    if vehicule.is_none() {
        return Err(e400().with_message("Vehicule with given id does not exist"))?;
    }
    let vehicule = vehicule.unwrap();

    if user.is_admin() {
        let api_response = ApiResponse::<Vehicule>::new()
            .with_message("Vehicule")
            .with_data(vehicule)
            .to_resp();
        
        return Ok(api_response);
    } else {
        let filtered_vehicule: Vehicule = if vehicule.is_available() && vehicule.is_active() {
            vehicule
        } else {
            return Err(e400().with_message("Vehicule with given id does not exist"))?;
        };
        let api_response = ApiResponse::<Vehicule>::new()
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
    if user.is_none() {
       return Err(e500())?; 
    }
   
    let user = user.unwrap();

    let vehicules = get_all_vehicules_sqlx(&pool).await
        .map_err(|_| e500())?;

    if user.is_admin() {
        let api_response = ApiResponse::<Vec<Vehicule>>::new()
            .with_message("List of vehicules")
            .with_data(vehicules)
            .to_resp();
        
        return Ok(api_response);
    } else {
        let filtered_vehicules: Vec<Vehicule> = vehicules
                       .into_iter().filter(|veh| veh.is_available() && veh.is_active())
                       .collect();
        let api_response = ApiResponse::<Vec<Vehicule>>::new()
            .with_message("List of vehicules")
            .with_data(filtered_vehicules)
            .to_resp();
        
        return Ok(api_response);
    }
}

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

#[tracing::instrument(
    name = "Delete vehicule query",
    skip(pool)
)]
async fn delete_vehicule_sqlx(
    pool: &PgPool,
    uuid: &Uuid,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        r#"
        DELETE FROM vehicules
        WHERE vehicule_id = $1
        "#,
        uuid
    )
    .execute(pool)
    .await;

    dbg!("{}", &query);

    //.context("Failed to execute query")?;

    if query?.rows_affected() == 0 {
        return Err(anyhow::anyhow!("Non existing vehicule"));
    }

    Ok(())
}

#[tracing::instrument(
    name = "Delete vehicule by id",
    skip(pool, session)
)]
pub async fn delete_vehicule(
    session: JwtSession,
    pool: web::Data<PgPool>,
    uuid: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = get_user_by_id(&pool, &session.user_id).await
        .map_err(|_| e500())?;
    if user.is_none() {
       return Err(e500())?; 
    }
    let user = user.unwrap();

    delete_vehicule_sqlx(&pool, &uuid).await
        .map_err(|_| e500())?;

    if !user.is_admin() {
        return Err(e403().with_message("You dont have required privilege"))?;
    }
    let api_response = ApiResponse::<()>::new()
        .with_message("Vehicule deleted")
        .to_resp();

    return Ok(api_response);
}
