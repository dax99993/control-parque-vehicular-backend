use actix_web::{HttpResponse, web};
//use secrecy::ExposeSecret;
use sqlx::PgPool;

use crate::authentication::jwt_session::{create_jwt, HmacKey};
use crate::error::error_chain_fmt;
use crate::models::user::LoginUser;
use crate::authentication::{Credentials, validate_credentials, AuthError};


#[tracing::instrument(
    name = "Login user",
    skip_all,
    fields(email=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login_user(
    pool: web::Data<PgPool>,
    body: web::Json<LoginUser>,
    key:  web::Data<HmacKey>,
) -> Result<HttpResponse, actix_web::Error> {
    let credentials = Credentials {
        email: body.0.email,
        password: body.0.password,
    };
    tracing::Span::current()
        .record("email", &tracing::field::display(&credentials.email));
    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            tracing::Span::current()
                .record("user_id", &tracing::field::display(&user_id));
            // Generate jwt
            let token: String = match create_jwt(&user_id, &key) {
                Ok(token) => token,
                Err(e) => {
                   tracing::error!("Couldn't create JWT {}", e);
                   /*
                   let e = LoginError::UnexpectedError(e.into());
                   return Err(login_error(e));
                   */
                   return Ok(HttpResponse::InternalServerError().json(
                           serde_json::json!({ "status": "failed", "message": "Couldnt create jwt" }))
                   );
                }
            };

            Ok(HttpResponse::Ok().json(
                    serde_json::json!({"status": "sucess", "token": token}))
            )

        },
        Err(e) => {
            let json = match e {
                /*
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
                */
                AuthError::InvalidCredentials(_) => serde_json::json!({ "status": "failed", "message": "Invalid credentials" }),
                AuthError::UnexpectedError(_) => serde_json::json!({ "status": "failed", "message": "Server Error" }),
            };
            return Ok(HttpResponse::InternalServerError().json(json));
        }
    }
}


#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error)
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
