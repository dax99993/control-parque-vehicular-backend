use actix_web::HttpResponse;

//use crate::authentication::middleware::JwtMiddleware;


pub async fn logout_user(
    //_: JwtMiddleware
) -> HttpResponse {
    //todo!()
    HttpResponse::Ok().json(
        serde_json::json!({"status": "success", "message": "You have logout"}))
}
