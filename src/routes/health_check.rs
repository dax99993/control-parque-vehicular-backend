use actix_web::{HttpResponse, Responder};

use crate::api_response::ApiResponse;

pub async fn health_check() -> Result<impl Responder, actix_web::Error> {
    Ok(ApiResponse::<()>::new().with_message("sucess"))
    //HttpResponse::Ok().finish()
}
