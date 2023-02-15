
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize>
{
    status: String,
    message: String,
    data: Option<T>
}

impl<T: Serialize> ApiResponse<T> {

}


#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub status: String,
    pub message: String,
}

impl std::fmt::Display for ApiErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}
