use std::{io::Error, net::TcpListener};

use crate::routes::{health_check, user};
use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

// Create HttpServer using actix-web
pub fn run(tcp_listener: TcpListener, connection_pool: PgPool) -> Result<Server, Error> {
    // Register connection pool as data
    let database_connection_pool = web::Data::new(connection_pool);
    // Create HttpServer instance
    let server = HttpServer::new(move || {
        // Create App instance
        App::new()
            .wrap(TracingLogger::default())
            .app_data(database_connection_pool.clone())
            // Register handler for GET /health_check
            .service(health_check)
            .configure(user::init_user_routes)
    })
    .listen(tcp_listener)?
    .run();

    // Return HttpServer instance
    Ok(server)
}
