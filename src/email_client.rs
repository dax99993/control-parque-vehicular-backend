
use secrecy::{Secret, ExposeSecret};
use lettre::transport::smtp::authentication::Credentials;
//use lettre::{SmtpClient, Transport};
use lettre_email::EmailBuilder;

pub struct EmailClient {
    smtp_address: String,
    username: String,
    password: Secret<String>,
}

impl EmailClient {
    pub fn new(
        smtp_address: String,
        username: String,
        password: Secret<String>,
    ) -> Self {
        Self {
            smtp_address,
            username,
            password,
        }
    }

    pub fn build_email(
        &self,
        to: String,
        subject: String,
        text: String,
        html: String,
    ) -> EmailBuilder {
       EmailBuilder::new()
           .from(self.username.clone())
           .to(to)
           .subject(subject)
           .text(text)
           .html(html)
    }



    
}

