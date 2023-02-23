use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use uuid::Uuid;

use crate::authentication::jwt_session::JwtSession;
use crate::api_response::{ApiResponse, e500, e403, e404};

use crate::routes::user::utils::get_user_by_id;


#[tracing::instrument(
    name = "Delete vehicule query",
    skip(pool)
)]
async fn delete_vehicule_sqlx(
    pool: &PgPool,
    uuid: &Uuid,
) -> Result<bool, sqlx::Error> {
    let query = sqlx::query!(
        r#"
        DELETE FROM vehicules
        WHERE vehicule_id = $1
        "#,
        uuid
    )
    .execute(pool)
    .await?;

    dbg!("{}", &query);

    Ok(query.rows_affected() != 0)
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
    let user = user.ok_or(e500())?;

    if !user.is_admin() {
        return Err(e403().with_message("You dont have required privilege"))?;
    }

    match delete_vehicule_sqlx(&pool, &uuid).await {
        Ok(deleted) => {
            if !deleted {
               return Err(e404().with_message("Vehicule not found"))?;
            }
        },
        Err(_) => { 
            return Err(e500())?;
        },
    }

    let api_response = ApiResponse::<()>::new()
        .with_message("Vehicule deleted")
        .to_resp();

    Ok(api_response)
}
