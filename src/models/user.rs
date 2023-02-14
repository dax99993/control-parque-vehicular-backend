use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
//use super::department::Department;
use validator::{Validate, ValidationError};
use secrecy::{Secret, ExposeSecret};


#[derive(sqlx::Type, Serialize, Deserialize, Debug, Clone)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Normal,
    Admin,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password_hash: String,
    pub employee_number: Option<i16>,
    pub active: bool,
    pub verified: bool,
    pub picture: String,
    //pub department: Department,
    pub department: Option<i32>,
    pub role: UserRole,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl User {
    /*
    pub fn new(first_name: String, last_name: String) -> Self {
        Self {
            id: 32,
            first_name,
            last_name,
            status: UserStatus::Normal,
            created_at: Some(Utc::now()),
            updated_at: None,
        }
    }
    */

    pub fn to_admin(&mut self) {
        self.role = UserRole::Admin;
    }
     
}


#[derive(Debug, Deserialize, Validate)]
pub struct SignupUser {
    #[validate(length(min = 1, max = 255))]
    pub first_name: String,
    #[validate(length(min = 1, max = 255))]
    pub last_name: String,
    #[validate(email)]
    pub email: String,
    //#[validate(length(min = 10, max = 255))]
    pub password: Secret<String>,
    //#[validate(length(min = 10, max = 255))]
    pub re_password: Secret<String>,
    //picture: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: Secret<String>,
}
