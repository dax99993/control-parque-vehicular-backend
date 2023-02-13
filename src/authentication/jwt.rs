use chrono::Utc;
use jsonwebtoken::{Header, Algorithm, encode, EncodingKey, decode, Validation, DecodingKey};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use secrecy::{Secret, ExposeSecret};

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
    pub fn new(user_id: &Uuid) -> Self {
        let issue_at = Utc::now().timestamp(); 

        let expiration = Utc::now()
            .checked_add_signed(chrono::Duration::hours(5))
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
