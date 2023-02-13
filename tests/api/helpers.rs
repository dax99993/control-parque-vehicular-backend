use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use once_cell::sync::Lazy;
//use wiremock::MockServer;
use control_parque_vehicular::configuration::{get_configuration, DatabaseSettings};
use control_parque_vehicular::email_client::EmailClient;
use control_parque_vehicular::startup::{Application, get_connection_pool};
use control_parque_vehicular::telemetry::{init_subscriber, get_subscriber};
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version};
//use control_parque_vehicular::issue_delivery_worker::{try_execute_task, ExecutionOutcome};

use fake::faker::internet::en::SafeEmail;
use fake::Fake;
//use fake::faker::name::en::Name;
//use fake::{Dummy, Faker, Fake};


// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

/// Confirmation links embedded in the request to the email API.
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    //pub email_server: MockServer,
    pub port: u16,
    pub test_user: TestUser,
    pub api_client: reqwest::Client,
    pub email_client: EmailClient,
}

impl TestApp {

    /*
    pub async fn dispatch_all_pending_emails(&self) {
        loop {
            if let ExecutionOutcome::EmptyQueue =
                try_execute_task(&self.db_pool, &self.email_client)
                    .await
                    .unwrap()
            {
                break;
            }
        }
    }
    */



    pub async fn post_change_password<Body>(&self, body: &Body) -> reqwest::Response 
        where
            Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/api/users/password", &self.address))
            .json(body)
            .send()
            .await
            .expect("failed to execute request")
    }

    /*
    pub async fn get_change_password(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/admin/password", &self.address))
            .send()
            .await
            .expect("failed to execute request")
    }

    pub async fn get_change_password_html(&self) -> String {
        self.get_change_password().await.text().await.unwrap()
    }

    pub async fn get_admin_dashboard(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/admin/dashboard", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_admin_dashboard_html(&self) -> String {
        self.get_admin_dashboard().await.text().await.unwrap()
    }

    pub async fn get_login_html(&self) -> String {
        self.api_client
            .get(&format!("{}/login", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
            .text()
            .await
            .unwrap()
    }
    */
    
    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response 
    where 
        Body: serde::Serialize
    {
        self.api_client
            .post(&format!("{}/api/auth/login", self.address))
            // makes sure that body is URL-encoded and `Content-Type` header is set accordingly
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/api/auth/logout", self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_register<Body>(&self, body: &Body) -> reqwest::Response
        where Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/api/auth/register", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    /*
    pub fn get_confirmation_links(
        &self,
        email_request: &wiremock::Request,
    ) -> ConfirmationLinks {
        // Parse the body as Json, starting from raw bytes
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        // Extract the link from one of the request fields
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            // Let's make sure we don't call random APIs on the web 
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            // Rewrite the URL to include the port
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };
        
        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(&body["TextBody"].as_str().unwrap());
        ConfirmationLinks {
            html,
            plain_text,
        }
    }
    
    
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type","application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_publish_newsletter<Body, Token>(&self, body: &Body, token: Token) -> reqwest::Response 
    where
        Body: serde::Serialize,
        Token: Display,
    {
        self.api_client
            .post(&format!("{}/admin/newsletters", &self.address))
            .bearer_auth(&self.test_user.username, Some(&self.test_user.password))
            .form(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    
    pub async fn get_publish_newsletter_html(&self) -> String {
        self.get_publish_newsletter().await.text().await.unwrap()
    }

    pub async fn get_publish_newsletter(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/admin/newsletters", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }
    */
}



// Launch our application in the background -somehow-
pub async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code `TRACING` is executed
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);
    
    //Launch a mock server to stand in for Postmark's API
    //let email_server = MockServer::start().await;

    // Randomise configuration to ensure test isolation
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        // Use a different database for each test case
        c.database.database_name = Uuid::new_v4().to_string();
        // Use a random OS port
        c.application.port = 0;
        // Use mock server as email API
        //c.email_client.base_url = email_server.uri();
        c
    };

    // Create and migrate the database
    configure_database(&configuration.database).await;

    // Lauch the application as a background task
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application");
    let application_port = application.port();
    let _ = tokio::task::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    let test_app = TestApp {
        address: format!("http://127.0.0.1:{}", application_port),
        db_pool: get_connection_pool(&configuration.database),
        //email_server,
        port: application_port,
        test_user: TestUser::generate(),
        api_client: client,
        email_client: configuration.email_client.client()
    };
    test_app.test_user.store(&test_app.db_pool).await;
    test_app
}


async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres Pool");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

pub struct TestUser {
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
       Self {
           user_id: Uuid::new_v4(),
           first_name: "first name".to_string(),
           last_name: "last name".to_string(),
           email: SafeEmail().fake(),
           password: Uuid::new_v4().to_string(),
       }
    }

    pub async fn store(&self, pool: &PgPool) {
        let salt = SaltString::generate(&mut rand::thread_rng());
        // We dont care about the exact Argon2 parameters here
        // given that it's for testing purposes
        let password_hash = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None).unwrap(),
            )
            .hash_password(self.password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        //dbg!(&password_hash);

        sqlx::query!(
            "INSERT INTO users (user_id, first_name, last_name, email, password_hash)
            VALUES ($1, $2, $3, $4, $5)",
            self.user_id,
            self.first_name,
            self.last_name,
            self.email,
            password_hash
            )
            .execute(pool)
            .await
            .expect("Failed to create test users.");
    }

    pub async fn login(&self, app: &TestApp) -> reqwest::Response {
        app.post_login(&serde_json::json!({
            "email": &self.email,
            "password": &self.password
        }))
        .await
    }
}

pub fn assert_is_a_redirect_to(response: &reqwest::Response, location: &str) {
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(response.headers().get("Location").unwrap(), location);
}
