use actix_web::{HttpResponse, web};

use crate::email_client::EmailClient;

pub async fn send_test_email(email_client: web::Data<EmailClient>) -> HttpResponse {
    let email_client = email_client.into_inner();

    match email_client.send_email(
        "dax99993@gmail.com",
        "Test Email",
        "<p><i>This is an email in html format</i></p>",
        "This is an email in plain_text format"
    )
    .await {
        Ok(_) => ok_response(),
        Err(_) => err_response(),
    }
}

fn ok_response() -> HttpResponse {
    HttpResponse::Ok().json(
        serde_json::json!({
            "status": "sucess", "message": "Email sent"
        }))
}

fn err_response() -> HttpResponse {
    HttpResponse::InternalServerError().json(
        serde_json::json!({
            "status": "failed", "message": "Email cannot be sent"
        }))
}
