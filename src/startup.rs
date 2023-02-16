
use std::net::TcpListener;

use crate::authentication::{jwt_session::HmacKey, middleware::reject_anonymous_user};
use crate::configuration::{Settings, DatabaseSettings};
use crate::email_client::EmailClient;
use actix_web::{web, App, HttpServer};
use actix_web::dev::Server;
use actix_web_lab::middleware::from_fn;
//use secrecy::{Secret, ExposeSecret};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::routes::{send_test_email, health_check, confirm, signup_user, login_user, logout_user};
use tracing_actix_web::TracingLogger;

//use redis::{Client, RedisResult};

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

//pub fn get_redis_client(configuration: &RedisClientSettings) -> {}


//#[derive(Debug)]
pub struct ApplicationBaseUrl(pub String);

pub struct RedisUri(pub String);

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        // redis_client
        //let redis_client = redis::Client::open(configuration.redis_client.uri.clone())?;
        let redis_uri = RedisUri(configuration.redis_client.uri);

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
                         configuration.application.base_url,
                         configuration.application.hmca_secret,
                         redis_uri,
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
    base_url: String,
    hmca_secret: HmacKey,
    //redis_client: redis::Client,
    redis_uri: RedisUri,
) -> Result<Server, anyhow::Error> {
    // Wrap data into smart pointer actix_web
    let db_pool = web::Data::new(db_pool);
    let hmca_secret = web::Data::new(hmca_secret);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    let redis_uri = web::Data::new(redis_uri);


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
                    .service(
                        web::scope("/auth")
                            .route("/signup", web::post().to(signup_user))
                            .route("/signups/confirm", web::get().to(confirm))
                            .route("/login", web::post().to(login_user))
                            .service(
                                web::resource("/logout")
                                    .wrap(from_fn(reject_anonymous_user))
                                    .route(web::get().to(logout_user))
                            )
                    )
                    .service(
                        web::scope("/users")
                            .wrap(from_fn(reject_anonymous_user))
                            .route("/me", web::get().to(health_check))
                    )
            )
            // Add all request extra data
            .app_data(db_pool.clone())
            .app_data(hmca_secret.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
            .app_data(redis_uri.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
