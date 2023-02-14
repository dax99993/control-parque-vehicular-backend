use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use uuid::Uuid;


#[derive(serde::Deserialize)]
pub struct Parameters {
    signup_token: String,
}

#[tracing::instrument(
    name = "Confirm a unverified user",
    skip(parameters, pool)
)]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    // TODO return response when user was already confirm
    let id = match get_user_id_from_token(&pool, &parameters.signup_token).await {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::InternalServerError().json(
                    serde_json::json!({"status": "fail", "message": "Server Error"})
                    );
        },
    };

    match id {
        None => {
            return HttpResponse::Unauthorized().json(
                    serde_json::json!({"status": "fail", "message": "Invalid token"})
                    );
        },
        Some(user_id) => {
            match confirm_user(&pool, user_id).await {
                Ok(_) => {
                    return HttpResponse::Ok().json(
                        serde_json::json!({"status": "sucess", "message": "User verified"})
                        );
                },
                Err(_) => {
                    return HttpResponse::InternalServerError().json(
                        serde_json::json!({"status": "fail", "message": "Server Error"})
                        );
                },
            }
        },
    }
    
}


#[tracing::instrument(
    name = "Get subscriber_id from token",
    skip(signup_token, pool)
)]
pub async fn get_user_id_from_token(
    pool: &PgPool,
    signup_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT user_id FROM  signup_tokens WHERE signup_token = $1"#,
        signup_token 
    )
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;
    Ok(result.map(|r| r.user_id))
}

#[tracing::instrument(
    name = "Mark user as verified",
    skip(user_id, pool)
)]
pub async fn confirm_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query!(
        r#"UPDATE users SET verified = true WHERE user_id = $1"#,
        user_id 
    )
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;
    Ok(())
}
