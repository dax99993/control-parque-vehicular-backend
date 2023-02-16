use actix_web::HttpResponse;

use crate::authentication::jwt_session::JwtSession;


pub async fn logout_user(
    jwt_session: JwtSession,
) -> Result<HttpResponse, actix_web::Error> {
    match jwt_session.blacklist_session() {
        Ok(_) => {},
        Err(_) => {
            return Ok(
                HttpResponse::Ok().json(
                    serde_json::json!({"status": "fail", "message": "This token has been logout already"}))
              );
        },
    }

    Ok(
    HttpResponse::Ok().json(
        serde_json::json!({"status": "success", "message": "You have logout"}))
    )
}
