use actix_web::{HttpResponse, web, ResponseError};
use sqlx::PgPool;
use uuid::Uuid;
use crate::error::error_chain_fmt;

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
    name = "Confirmar usuario sin verificar",
    skip(parameters, pool)
)]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {

    if let Some(usuario_id) = 
        obtener_usuario_id_del_token_sqlx(&pool, &parameters.signup_token).await
            .map_err(|e| VerifyError::UnexpectedError(e.into()))?
    {
        // No verificar usuario si ya esta verificado
        let verificado = obtener_campo_verificado_sqlx(&pool, usuario_id).await
            .map_err(|e| VerifyError::UnexpectedError(e.into()))?;

        if verificado {
            return Err(VerifyError::AlreadyVerifiedUserError("".into()))?;
        }

        verificar_usuario_sqlx(&pool, usuario_id).await
            .map_err(|e| VerifyError::UnexpectedError(e.into()))?;

        return Ok(HttpResponse::Ok().json(
            serde_json::json!({"status": "exito", "message": "Usuario verificado"})
            ));
            
    } else {
        return Err(VerifyError::InvalidTokenError("".into()))?;
    }


}


#[tracing::instrument(
    name = "Get subscriber_id from token",
    skip(signup_token, pool)
)]
pub async fn obtener_usuario_id_del_token_sqlx(
    pool: &PgPool,
    signup_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT usuario_id FROM  signup_tokens WHERE signup_token = $1"#,
        signup_token 
    )
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;

    Ok(result.map(|r| r.usuario_id))
}

#[tracing::instrument(
    name = "Verificar usuario",
    skip(usuario_id, pool)
)]
pub async fn verificar_usuario_sqlx(
    pool: &PgPool,
    usuario_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE usuarios
        SET verificado = true,
            modificado_en = now()
        WHERE usuario_id = $1"#,
        usuario_id 
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
    name = "Obtener campo verificado del usuario",
    skip(usuario_id, pool)
)]
pub async fn obtener_campo_verificado_sqlx(
    pool: &PgPool,
    usuario_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT verificado
        FROM usuarios
        WHERE usuario_id = $1"#,
        usuario_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(row.verificado)
}
