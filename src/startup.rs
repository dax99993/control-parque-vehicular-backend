
use std::net::TcpListener;

use crate::configuration::{Settings, DatabaseSettings};
use actix_web::{web, App, HttpServer};
use actix_web::dev::Server;
//use secrecy::{Secret, ExposeSecret};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::routes::health_check;


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
        //

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
                         //email_client,
                         //configuration.application.base_url,
                         //configuration.application.hmac_secret,
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
    //email_client: ,
    //base_url: String,
    //hmac_secret: Secret<String>,
    //redis_uri: Secret<String>,
) -> Result<Server, anyhow::Error> {
    // Wrap data into smart pointer actix_web
    let db_pool = web::Data::new(db_pool);

    // Create the server
    let server = HttpServer::new(move || {
        App::new()
            // Add API service
            .service(
                web::scope("/api")
                    // wrap all middleware
                    //.wrap()
                    // Add routes
                    .route("/health-check", web::get().to(health_check))
                    /*
                    .route("/login", web::get().to())
                    .route("/login", web::post().to())
                    */
            )
            // Add all request extra data
            .app_data(db_pool.clone())
            //.app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
