use super::jwt_session::JwtSession;

use actix_web_lab::middleware::Next;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::FromRequest;
use actix_web::HttpMessage;


pub async fn reject_anonymous_user(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let session = { 
        let (http_request, payload) = req.parts_mut();
        JwtSession::from_request(http_request, payload).await?
    };

    req.extensions_mut()
        .insert::<JwtSession>(session);

    next.call(req).await
}
