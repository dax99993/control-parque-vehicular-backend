mod health_check;
mod email_check;
mod auth;
pub mod user;
mod images;
mod department;
mod vehicules;

pub use health_check::*;
pub use email_check::send_test_email;
pub use auth::register::signup_user;
pub use auth::login::login_user;
pub use auth::logout::logout_user;
pub use auth::signup_confirm::confirm;
pub use user::user_get_me;
pub use user::{users_get_all, users_get_user_by_id, users_delete_user_by_id, user_patch};
pub use images::get_image;
pub use department::*;
pub use vehicules::*;
