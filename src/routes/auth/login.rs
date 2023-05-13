use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use crate::api_response::{ApiResponse, e500, e401};
use crate::authentication::jwt_session::{create_jwt, HmacKey};
use crate::authentication::{Credentials, validate_credentials, AuthError};


#[tracing::instrument(
    name = "Login user",
    skip_all,
    fields(email=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login_user(
    pool: web::Data<PgPool>,
    body: web::Json<Credentials>,
    key:  web::Data<HmacKey>,
) -> Result<HttpResponse, actix_web::Error> {

    // Convertir json a credentials
    let credentials = body.0;

    // Log email
    tracing::Span::current()
        .record("email", &tracing::field::display(&credentials.email));

    // Validar credenciales 
    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            tracing::Span::current()
                .record("usuario_id", &tracing::field::display(&user_id));
            // Generar jwt
            let token: String = match create_jwt(&user_id, &key) {
                Ok(token) => token,
                Err(e) => {
                   tracing::error!("No se pudo crear el JWT {}", e);
                   return Err(e500())?;
                }
            };

            // Respuesta exitosa
            let api_response = ApiResponse::<String>::new()
               .with_message("Token creaado")
               .with_data(token)
               .to_resp();

            Ok(api_response)
        },
        Err(e) => {
            let api_response =  match e {
                AuthError::InvalidCredentials(_) => e401().with_message("credenciales invalidas"),
                AuthError::UnexpectedError(_) => e500(),
            };
            Err(api_response)?
        }
    }
}
