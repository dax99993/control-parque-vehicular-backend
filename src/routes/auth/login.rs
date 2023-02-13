use actix_web::error::InternalError;
use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use crate::authentication::jwt::{create_jwt, HmacKey};
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
    //_session: JwtMiddleware,
) -> Result<HttpResponse, InternalError<LoginError>> {
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
            let token: String = create_jwt(&user_id, &key)
                .or_else(|e| {
                    tracing::error!("Couldn't create JWT");
                   let e = LoginError::UnexpectedError(e.into());
                   return Err(login_error(e));
                })
            .unwrap();

            Ok(HttpResponse::Ok().json(
                    serde_json::json!({"status": "sucess", "token": token}))
            )

        },
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };
            Err(login_error(e))
        }
    }
}

fn login_error(e: LoginError) -> InternalError<LoginError> {
    let response = HttpResponse::InternalServerError().finish();
    InternalError::from_response(e, response)
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
