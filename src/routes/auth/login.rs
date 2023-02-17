use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use crate::api_response::{ApiResponse, e500, e401};
use crate::authentication::jwt_session::{create_jwt, HmacKey};
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
                   return Err(e500())?;
                }
            };

            Ok(ApiResponse::<_>::new().with_message("Token created").with_data(token).to_resp())
        },
        Err(e) => {
            let api_response =  match e {
                AuthError::InvalidCredentials(_) => e401().with_message("Invalid credentials"),
                AuthError::UnexpectedError(_) => e500(),
            };
            Err(api_response)?
        }
    }
}
