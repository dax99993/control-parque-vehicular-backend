use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
//use super::department::Department;
//use validator::{Validate, ValidationError};
//use secrecy::{Secret, ExposeSecret};
use validator::Validate;
use secrecy::Secret;


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct User {
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password_hash: String,
    pub employee_number: Option<i16>,
    pub active: bool,
    pub verified: bool,
    pub picture: String,
    pub department: Option<i32>,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.role == "admin".to_string()
    }

    pub fn is_normal(&self) -> bool {
        self.role == "normal".to_string()
    }

    pub fn update(mut self, user: UpdateUser) -> Self {
        self.first_name =  user.first_name.unwrap_or_else(|| self.first_name);
        self.last_name = user.last_name.unwrap_or_else(|| self.last_name);
        self.employee_number = user.employee_number.unwrap_or_else(|| self.employee_number); 
        self.active = user.active.unwrap_or_else(|| self.active); 
        self.verified = user.verified.unwrap_or_else(|| self.verified); 
        self.department = user.department.unwrap_or_else(|| self.department);
        self.role = user.role.unwrap_or_else(|| self.role);
        //self.email = user.email.unwrap_or_else(|| self.email);
        //self.picture: user.picture,

        self
    }

    pub fn update_me(mut self, user: UpdateUserMe) -> Self {
        self.first_name =  user.first_name.unwrap_or_else(|| self.first_name);
        self.last_name = user.last_name.unwrap_or_else(|| self.last_name);
        self.employee_number = user.employee_number.unwrap_or_else(|| self.employee_number); 
        self.department = user.department.unwrap_or_else(|| self.department);
        //self.email = user.email.unwrap_or_else(|| self.email);
        //self.picture: user.picture,

        self
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
}

#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: Secret<String>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FilteredUser {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub employee_number: Option<i16>,
    pub department: Option<i32>,
    pub picture: String,
}

impl FilteredUser {
    pub fn from(user: User) -> Self {
        Self { 
            first_name: user.first_name,
            last_name: user.last_name,
            email: user.email,
            employee_number: user.employee_number,
            picture: user.picture,
            department: user.department
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateUser {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub employee_number: Option<Option<i16>>,
    pub active: Option<bool>,
    pub verified: Option<bool>,
    //pub picture: Option<String>,
    pub department: Option<Option<i32>>,
    pub role: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateUserMe {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub employee_number: Option<Option<i16>>,
    //pub email: Option<String>,
    //pub picture: Option<String>,
    pub department: Option<Option<i32>>,
}

/*
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HyperlinkUser {
    pub user_id: String,
    pub first_name: str,
    pub last_name: String,
    pub email: String,
    pub password_hash: String,
    pub employee_number: Option<i16>,
    pub active: bool,
    pub verified: bool,
    pub picture: String,
    pub department: Option<String>,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    #[serde(skip_deserializing)]
    pub base_url: String,
    #[serde(skip_deserializing)]
    pub user: User,
}

impl HyperlinkUser {
    pub fn build(mut self) -> Self {
        self.user_id = format!("{}/api/users/{}", self.base_url, self.user.user_id);
        self.first_name = self.user.first_name;
        self.last_name = self.user.last_name;
        self.email = self.user.email;
        self.password_hash = self.user.password_hash;
        self.employee_number = self.user.employee_number;
        self.active = self.user.active;
        self.verified = self.user.verified;
        self.picture = format!("{}/api/images/users/{}", self.base_url, self.user.picture);
        if self.user.department.is_some() {
            self.department = Some(format!("{}/api/deparments/{}", self.base_url, self.user.department.unwrap()));
        } else {
            self.department = None;
        }
        self.role = self.user.role;
        self.created_at = self.user.created_at;
        self.updated_at= self.user.updated_at;

        self
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
    pub fn with_user(mut self, user: User) -> Self {
        self.user = user;
        self
    }

}
*/
