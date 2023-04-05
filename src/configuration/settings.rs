use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde_aux::prelude::deserialize_number_from_string;
use sqlx::ConnectOptions;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use tracing_log::log;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
}

#[derive(Debug, Deserialize)]
pub struct ApplicationSettings {
    pub host_address: String,
    // Standard serde will fail to pick up integer from config
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    pub database_name: String,
    pub host: String,
    // Save password with secret
    pub password: Secret<String>,
    // Standard serde will fail to pick up integer from config
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub username: String,
    // For ssl mode
    pub ssl_mode: bool,
}

impl DatabaseSettings {
    // Get without database
    pub fn without_database(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.username)
            .password(self.password.expose_secret())
            .ssl_mode(if self.ssl_mode {
                PgSslMode::Require
            } else {
                PgSslMode::Prefer
            })
    }

    // Get with database
    pub fn with_database(&self) -> PgConnectOptions {
        let mut options = self.without_database().database(&self.database_name);
        options.log_statements(log::LevelFilter::Trace);
        options
    }
}
