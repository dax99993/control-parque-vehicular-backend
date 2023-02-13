//use crate::domain::SubscriberEmail;
use secrecy::{Secret, ExposeSecret};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, AsyncSmtpTransport, Address, Tokio1Executor, AsyncTransport};
use lettre::message::{Mailbox, header};


pub struct EmailClient {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}


impl EmailClient {
    pub fn new(smtp_address: String,
               smtp_name: String,
               smtp_username: String,
               smtp_password: Secret<String>,
               smtp_port: u64,
    ) -> Self {
        let _credentials = Credentials::new(smtp_username.clone(), format!("{}", smtp_password.expose_secret()));
        
        /*
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(smtp_address.as_ref())
            .unwrap()
            .credentials(credentials)
            .build();
        */
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(smtp_address)
            .port(smtp_port as u16)
            .build();

        let split = smtp_username.split("@").collect::<Vec<&str>>();
        let (user, domain) = (split.get(0).unwrap(), split.get(1).unwrap());

        let address = Address::new(user, domain).unwrap();
        let from = Mailbox::new(Some(smtp_name), address);

        Self {
            mailer,
            from
        }
    }

    pub async fn send_email(
        &self,
        recipient: &str,
        subject: &str,
        html_content: &str,
    ) -> Result<(), <AsyncSmtpTransport::<Tokio1Executor> as AsyncTransport>::Error> {

        // Create email
        let recipient = format!("<{}>", recipient);
        let email = Message::builder()
            .from(self.from.clone())
            .to(recipient.parse().unwrap())
            .subject(subject)
            .header(header::ContentType::TEXT_HTML)
            .body(format!("{}", html_content))
            .unwrap();

        // Send email
        match self.mailer.send(email).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

/*
// Based on Postmark's API
#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use wiremock::matchers::{any, header, path, method, header_exists};
    use wiremock::{Mock, MockServer, ResponseTemplate, Request};
    use secrecy::Secret;
    use claims::{assert_err, assert_ok};

    /// Generate a random email subject
    fn subject() -> String {
        Sentence(1..2).fake()
    }
    
    /// Generate a random email content
    fn content() -> String {
        Paragraph(1..10).fake()
    }

    /// Generate a random subscriber email 
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    /// Get a test instance of `EmailClient`
    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(base_url,
                         email(),
                         Secret::new(Faker.fake()),
                         std::time::Duration::from_millis(200),
                        )
    }


    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            // Try to parse the body as a JSON value
            let result: Result<serde_json::Value, _> =
                serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                // check that all the mandatory fields are populated
                // without inspecting the field values
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                // If parsing failed, do not match the request
                false
            }
        }
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let _ = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        // Assert
        // Mock asserts on drop
    }

    // Happy-path test
    #[tokio::test]
    async fn send_email_succeds_if_the_server_returns_200() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        // We add the bare minimum needed to trigger the path we want
        // to test in `send_email`
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;


        // Act
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        // Assert
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;


        // Act
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        // Assert
        assert_err!(outcome);
    }


    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        let response = ResponseTemplate::new(200)
            .set_delay(std::time::Duration::from_secs(180));

        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;


        // Act
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        // Assert
        assert_err!(outcome);
    }
}
*/
