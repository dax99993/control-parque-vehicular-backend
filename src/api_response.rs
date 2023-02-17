use actix_web::{ResponseError, HttpResponse, HttpResponseBuilder, Responder, body::BoxBody};
use serde::{Serialize, Deserialize};
use actix_web::http::StatusCode;
use std::borrow::Cow;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T= ()>
{
    #[serde(skip_serializing)]
    status_code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>
}
pub type ApiError = ApiResponse<()>;


impl<T: Serialize + std::fmt::Debug> std::fmt::Display for ApiResponse<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

impl<T: Serialize + std::fmt::Debug> ApiResponse<T> {
    pub fn new() -> Self {
        Self { 
            status_code: 200,
            status: Some("success".into()),
            //status: None,
            message: None,
            data: None
        }
    }

    pub fn with_status_code(mut self, status_code: u16) -> Self {
        self.status_code = status_code;
        self
    }


    pub fn with_status<S: Into<Cow<'static, str>>>(mut self, status: S) -> Self { 
        self.status= Some(status.into());
        self
    }

    pub fn with_message<S: Into<Cow<'static, str>>>(mut self, message: S) -> Self { 
        self.message = Some(message.into());
        self
    }

    pub fn with_data(mut self, data: T) -> Self {
        self.data = Some(data);
        self
    }

    pub fn to_resp(&self) -> HttpResponse {
        //let body = serde_json::json!()
        HttpResponseBuilder::new(self.status_code())
            .json(self)
        
    }
}

impl<T: Serialize + std::fmt::Debug> ResponseError for ApiResponse<T> {
    fn status_code(&self) -> StatusCode {
        match StatusCode::from_u16(self.status_code) {
            Ok(status) => status,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponseBuilder::new(self.status_code())
            .json(self)
    }
}


impl<T: Serialize + std::fmt::Debug>  Responder for ApiResponse<T> {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        self.to_resp()
        //HttpResponseBuilder::new(self.status_code())
        //    .json(self)
            
    }
}

pub fn e500() -> ApiError {
    ApiError::new()
            .with_status_code(500)
            .with_status("fail")
            .with_message("Server Error")
}

pub fn e401() -> ApiError {
    ApiError::new()
            .with_status_code(401)
            .with_status("fail")
}

pub fn e409() -> ApiError {
    ApiError::new()
            .with_status_code(409)
            .with_status("fail")
}
