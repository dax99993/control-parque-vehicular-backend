use chrono::Utc;
use jsonwebtoken::{Header, Algorithm, encode, EncodingKey, decode, Validation, DecodingKey};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use secrecy::{Secret, ExposeSecret};

use actix_web::{HttpRequest, web, FromRequest};
use actix_web::http;
use actix_web::dev::Payload;
use std::future::{ready, Ready};

use crate::api_response::{e401, e500};
use crate::startup::RedisUri;

use redis::Commands;



#[derive(Debug, Clone, Deserialize)]
pub struct HmacKey(pub Secret<String>);

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
//pub struct JwtSession<'a> {
pub struct JwtSession {
    pub user_id: Uuid,
    pub token: String,
    pub redis_client: redis::Client
    //pub redis_client: &'a redis::Client
    //pub redis_con: redis::aio::ConnectionManager
}

//impl<'a> JwtSession<'a> {
impl JwtSession {
    pub fn new(user_id: Uuid, token: String, redis_client: redis::Client) -> Self {
        Self { user_id, token, redis_client}
    }

    pub fn get_blacklist_key(&self) -> String {
        format!("user.id:{}:blacklist.tokens", self.user_id)
    }

    #[tracing::instrument(
    name = "Blacklist current JWT session",
    skip(self)
    )]
    pub fn blacklist_session(&self) -> Result<(), anyhow::Error> {
        let mut redis_con = self.redis_client.get_connection().unwrap();

        let result: Result<(), redis::RedisError> = redis::cmd("SADD")
            .arg(self.get_blacklist_key())
            .arg(&self.token)
            .query(&mut redis_con);
        //map error
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(e.into())
        }
    }

    #[tracing::instrument(
    name = "Check if JWT Session is blacklisted",
    skip(self)
    )]
    pub fn is_blacklisted(&self) -> Result<bool, anyhow::Error> {
        let mut redis_con = self.redis_client.get_connection().unwrap();
        match redis_con.sismember(self.get_blacklist_key(), &self.token) {
            Ok(blacklisted) => { Ok(blacklisted) },
            Err(e) => { Err(e.into()) },
        }
    }
}

impl FromRequest for JwtSession {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let hmca_key = if let Some(key) = req.app_data::<web::Data<HmacKey>>() {
            key.get_ref()
        } else {
            return ready(Err(e500().into()))
        };


        let redis_uri= if let Some(uri) =  req.app_data::<web::Data<RedisUri>>() {
            uri.get_ref()
        } else {
            return ready(Err(e500().into()))
        };

        let redis_client = match redis::Client::open(redis_uri.0.clone()) {
            Ok(client) => client,
            Err(_) =>  return ready(Err(e500().into())),
        };


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
            return ready(Err(e401().with_message("Please provide a token").into()))
        }
        
        let claims = match TokenClaims::from_token(token.unwrap(), hmca_key) {
            Ok(c) => c,
            Err(_) => {
                return ready(Err(e401().with_message("Invalid token").into()))
            }
        };


        let user_id = uuid::Uuid::parse_str(claims.sub.as_str()).unwrap();
        let jwt_session = JwtSession::new(user_id, token.unwrap().to_string(), redis_client);
        
        // Check blacklist tokens
        match jwt_session.is_blacklisted() {
            Ok(exists) => {
                if exists {
                    return ready(Err(e401().with_message("Blacklisted token").into()))
                }
            },
            Err(_) => {
                return ready(Err(e500().into()))
            },
        }

        ready(Ok(jwt_session))
    }
}
