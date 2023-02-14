
use std::net::TcpListener;

use crate::authentication::jwt::HmacKey;
use crate::configuration::{Settings, DatabaseSettings};
use crate::email_client::EmailClient;
use actix_web::{web, App, HttpServer};
use actix_web::dev::Server;
use secrecy::{Secret, ExposeSecret};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::routes::{send_test_email, health_check, confirm, signup_user, login_user, logout_user};
use tracing_actix_web::TracingLogger;


pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}


pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        // email client
        let email_client = EmailClient::new(
            configuration.email_client.smtp_host,
            configuration.email_client.smtp_name,
            configuration.email_client.smtp_username,
            configuration.email_client.smtp_password,
            configuration.email_client.smtp_port
            )?;

        let address = format!(
            "{}:{}",
            configuration.application.host,
            configuration.application.port,
        );
        // if port == 0 then the OS will assign an available port (this is used in tests)
        let listener = TcpListener::bind(address)
            .expect("Failed to bind address");
        // Here we are getting the assign port when doing test
        let port = listener.local_addr().unwrap().port();

        let server = run(listener,
                         connection_pool,
                         email_client,
                         //configuration.application.base_url,
                         configuration.application.hmca_secret,
                         //configuration.redis_uri,
                    ).await?;
        Ok( Self { port, server } )
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}


async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    //base_url: String,
    hmca_secret: HmacKey,
    //redis_uri: Secret<String>,
) -> Result<Server, anyhow::Error> {
    // Wrap data into smart pointer actix_web
    let db_pool = web::Data::new(db_pool);
    let hmca_secret = web::Data::new(hmca_secret);
    let email_client = web::Data::new(email_client);
    //let _secret_key = jwt_secret.expose_secret();

    // Create the server
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            // Add API service
            .service(
                web::scope("/api")
                    // wrap all middleware
                    //.wrap()
                    // Add routes
                    .route("/health-check", web::get().to(health_check))
                    .route("/email-check", web::get().to(send_test_email))
                    /*
                    .route("/login", web::get().to())
                    .route("/login", web::post().to())
                    */
                    .route("/auth/signup", web::post().to(signup_user))
                    .route("/auth/signups/confirm", web::get().to(confirm))
                    .route("/auth/login", web::post().to(login_user))
                    .route("/auth/logout", web::get().to(logout_user))
            )
            // Add all request extra data
            .app_data(db_pool.clone())
            .app_data(hmca_secret.clone())
            .app_data(email_client.clone())
            //.app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
