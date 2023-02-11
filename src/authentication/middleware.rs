use std::future::Ready;

use actix_web::{HttpRequest, web, HttpMessage, FromRequest};
use actix_web::http::header::AUTHORIZATION;
use actix_web::error::ErrorUnauthorized;
use actix_web::dev::Payload;

use uuid::Uuid;


#[derive(Debug, Clone)]
pub struct JwtMiddleware(Uuid);


impl FromRequest for JwtMiddleware {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
       todo!() 
    }
}
