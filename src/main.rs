use std::{io::Error, net::TcpListener, time::Duration};

use actix_template::{configuration, startup, telemetry};
use sqlx::postgres::PgPoolOptions;

#[actix_web::main]
async fn main() -> Result<(), Error> {
    // Setup logger
    telemetry::init_subscriber("actix-template".into(), "info".into());
    let configuration = configuration::get_configuration().expect("Failed to read configuration.");

    // Setup PostgreSQL connection pool
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_database());

    // Create Tcp listener
    let address = format!(
        "{}:{}",
        configuration.application.host_address, configuration.application.port
    );
    let app_listener = TcpListener::bind(address)?;
    // Start HTTP server
    startup::run(app_listener, connection_pool)?.await
}
