use crate::email_client::EmailClient;
use secrecy::{Secret, ExposeSecret};
use serde_aux::prelude::deserialize_number_from_string;
use sqlx::ConnectOptions;
use sqlx::postgres::{PgSslMode, PgConnectOptions};


#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings// email client
    // shared cache redis
}


#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub base_url: String,
    pub jwt_secret: Secret<String>,
}

#[derive(serde::Deserialize, Clone)]
pub struct EmailClientSettings {
    pub smtp_address: String,
    pub username: String,
    pub password: Secret<String>,
}

impl EmailClientSettings {
    pub fn client(self) -> EmailClient {
        EmailClient::new(
            self.smtp_address,
            self.username,
            self.password,
        )
    }
}



#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            // Try an encrypted connection, fallback to unencrypted if it fails
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        let mut options = self.without_db().database(&self.database_name);
        options.log_statements(tracing::log::LevelFilter::Trace);

        options
    }
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production=> "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
       match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                    "{} is not a supported environment. Use either `local` or `production`.",
                    other
            ))
       }
    }
}


pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine current directory");
    let configuration_directory = base_path.join("configuration");
    // Add configuration values from a file named 'configuration'
    // It will look for any top-level file with an extension
    // that 'config' knows how to parse: yaml, json, etc.

    // Detect the running environment
    // Default to `local` if unspecified
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    let settings = config::Config::builder()
        // Read the default configuration
        .add_source(config::File::from(configuration_directory.join("base")).required(true))
        // Layer on the environment-specific values
        .add_source(config::File::from(configuration_directory.join(environment.as_str())).required(true))
        // Add in settings from environment variables (with a prefix of APP and '__' as separator)
        // E.g `APP_APPLICATION__PORT=4001` would set `Settings.application.port`
        .add_source(config::Environment::with_prefix("app").separator("__"))
        .build()
        .unwrap()
        .try_deserialize::<Settings>();
        
    settings
}

