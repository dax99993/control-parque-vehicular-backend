use chrono::{NaiveDate, DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct User {
    pub id: uuid::Uuid,
    pub first_name: String,
    pub last_name: String,
    pub privilage: UserPrivilage,
    //created_at: NaiveDate,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl User {
    pub fn new(first_name: String, last_name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            first_name,
            last_name,
            privilage: UserPrivilage::Normal,
            created_at: Some(Utc::now()),
            updated_at: None,
        }
    }

    pub fn to_admin(&mut self) {
        self.privilage = UserPrivilage::Admin;
    }
     
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub enum UserPrivilage {
    Normal,
    Admin,
}
