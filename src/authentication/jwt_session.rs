use chrono::Utc;
use jsonwebtoken::{Header, Algorithm, encode, EncodingKey, decode, Validation, DecodingKey};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use secrecy::{Secret, ExposeSecret};

use actix_web::{HttpRequest, web, HttpMessage, FromRequest};
use actix_web::http;
use actix_web::error::ErrorUnauthorized;
use actix_web::dev::Payload;
use std::future::{ready, Ready};

use crate::error::error_chain_fmt;

#[derive(Debug, Clone, Deserialize)]
pub struct HmacKey(pub Secret<String>);


#[derive(thiserror::Error)]
enum JwtError {
    #[error("{0}")]
    TokenCreationError(String),
    #[error(transparent)]
    TokenReadingError(#[from] anyhow::Error),
}

impl std::fmt::Debug for JwtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}


impl TokenClaims {
    const EXPIRATION_HOURS: i64 = 5;

    pub fn new(user_id: &Uuid) -> Self {
        let issue_at = Utc::now().timestamp(); 

        let expiration = Utc::now()
            .checked_add_signed(chrono::Duration::hours(Self::EXPIRATION_HOURS))
            .expect("valid timestamp")
            .timestamp();

        Self {
            sub: user_id.to_string(),
            iat: issue_at as usize,
            exp: expiration as usize,
        }
    }

    
    pub fn from_token(token: &str, hmac_key: &HmacKey) -> Result<Self, jsonwebtoken::errors::Error> {
        let validation = Validation::new(Algorithm::HS256);
        let key = DecodingKey::from_secret(hmac_key.0.expose_secret().as_bytes());
        let token_data = decode::<Self>(token, &key, &validation)?;

        Ok(token_data.claims)
    }

    pub fn get_user_id(&self) -> Result<Uuid, uuid::Error> {
        Uuid::parse_str(self.sub.as_str()) 
    }
    
}

#[tracing::instrument(name = "Create JWT", skip(key))]
pub fn create_jwt(user_id: &Uuid, key: &HmacKey) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = TokenClaims::new(user_id);
    let header = Header::new(Algorithm::HS256);
    encode(&header,
           &claims,
           &EncodingKey::from_secret(key.0.expose_secret().as_bytes())
    )
}



#[derive(Debug, Clone)]
pub struct JwtSession(Uuid);

impl JwtSession {
    pub fn get_user_id(&self) -> Uuid {
        self.0
    }
}

impl FromRequest for JwtSession {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let hmca_key = req.app_data::<web::Data<HmacKey>>()
            .unwrap()
            //.expect("No key available")
            .get_ref();

        let token = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| {
                let words = h.split("Bearer").collect::<Vec<&str>>();
                let token = words.get(1).map(|w| w.trim());

                token
            });
            /*
            .or_else(|| {
                req.cookie("token")
                    .map(|c| c.value())
            });
            */

        if token.is_none() {
            let json_error = serde_json::json!({
                "status": "failed".to_string(),
                "message": "You are not logged in, please provide token".to_string(),
            });
            return ready(Err(ErrorUnauthorized(json_error)));
        }
        
        let claims = match TokenClaims::from_token(token.unwrap(), hmca_key) {
            Ok(c) => c,
            Err(_) => {
                let json_error = serde_json::json!({
                    "status": "failed".to_string(),
                    "message": "Invalid token".to_string(),
                });
                return ready(Err(ErrorUnauthorized(json_error)));
            }
        };

        let user_id = uuid::Uuid::parse_str(claims.sub.as_str()).unwrap();
        //let user_id = claims.get_user_id().unwrap();
        //req.extensions_mut()
        //    .insert::<uuid::Uuid>(user_id.to_owned());

        ready(Ok(JwtSession(user_id)))
    }
}
