
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse {
    status: String,
    message: String,
    data: Option<String>
}


