mod health_check;
mod email_check;
mod images;

pub mod department;
pub mod auth;
pub mod users;
pub mod vehicules;
pub mod requests;

pub mod struct_check;


pub use health_check::*;
pub use email_check::send_test_email;
pub use images::get_image;
