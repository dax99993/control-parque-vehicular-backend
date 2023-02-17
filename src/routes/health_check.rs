use actix_web::HttpResponse;

//use crate::api_response::ApiResponse;

pub async fn health_check() -> Result<HttpResponse, actix_web::Error> {
    //Ok(ApiResponse::<()>::new().with_message("sucess"))
    Ok(HttpResponse::Ok().finish())
}
