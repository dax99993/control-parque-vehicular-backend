use crate::error::error_chain_fmt;
use actix_web::{HttpResponse, web, ResponseError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(thiserror::Error)]
pub enum VerifyError {
    #[error("{0}")]
    InvalidTokenError(String),
    #[error("{0}")]
    AlreadyVerifiedUserError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for VerifyError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self {
            Self::AlreadyVerifiedUserError(_) => {
                HttpResponse::Conflict().json(
                        serde_json::json!({"status": "fail", "message": "Account was already verified"})
                        )
            },
            Self::InvalidTokenError(_) => {
                HttpResponse::BadRequest().json(
                serde_json::json!({"status": "fail", "message": "Invalid token"})
                )
            },
            Self::UnexpectedError(_) => {
                HttpResponse::InternalServerError().json(
                    serde_json::json!({"status": "fail", "message": "Server Error"})
                    )
            },
        }
    }
}


#[derive(serde::Deserialize)]
pub struct Parameters {
    signup_token: String,
}

#[tracing::instrument(
    name = "Confirm an unverified user",
    skip(parameters, pool)
)]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    if let Some(user_id) = 
        get_user_id_from_token(&pool, &parameters.signup_token).await
            .map_err(|e| VerifyError::UnexpectedError(e.into()))?
    {
        // Dont confirm user if it was already confirmed
        let verified = get_verified_field(&pool, user_id).await
            .map_err(|e| VerifyError::UnexpectedError(e.into()))?;
        if verified {
            return Err(VerifyError::AlreadyVerifiedUserError("".into()))?;
        }

        confirm_user(&pool, user_id).await
            .map_err(|e| VerifyError::UnexpectedError(e.into()))?;

            return Ok(HttpResponse::Ok().json(
                serde_json::json!({"status": "sucess", "message": "User verified"})
                ));
            
    } else {
        return Err(VerifyError::InvalidTokenError("".into()))?;
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
    sqlx::query!(
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

#[tracing::instrument(
    name = "Get user verified field",
    skip(user_id, pool)
)]
pub async fn get_verified_field(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT verified
        FROM users
        WHERE user_id = $1"#,
        user_id 
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(row.verified)
}
