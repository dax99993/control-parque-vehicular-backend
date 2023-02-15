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

    let user_id = session.get_user_id();
    req.extensions_mut()
        .insert::<uuid::Uuid>(user_id.to_owned());

    next.call(req).await
}
