use chrono::{NaiveDateTime};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Department {
    pub id: i32,
    pub name: String,
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug, Clone)]
#[sqlx(type_name = "user_status", rename_all = "lowercase")]
pub enum UserStatus {
    Normal,
    Admin,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub employee_number: Option<i16>,
    pub active: bool,
    pub picture: String,
    //pub department: Department,
    pub department: Option<i32>,
    pub status: UserStatus,
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
        self.status = UserStatus::Admin;
    }
     
}

