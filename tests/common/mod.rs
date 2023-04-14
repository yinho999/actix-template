use {{project-name}}::telemetry::init_subscriber;
use {{project-name}}::{get_configuration, telemetry, DatabaseSettings};
use once_cell::sync::Lazy;
use sqlx::{Executor, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}
// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber_name = "{{project-name}}-test".to_string();
    let subscriber_filter = "debug".to_string();

    // Do not show the `tracing` logs in the test output unless env var `TEST_LOG` is set
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, subscriber_filter, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, subscriber_filter, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let address = format!("http://{}", listener.local_addr().unwrap());
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let db_pool = configure_test_database(&configuration.database).await;
    let server = {{project-name}}::run(listener, db_pool.clone()).expect("Failed to bind address");
    actix::spawn(server);
    TestApp { address, db_pool }
}

async fn configure_test_database(configuration: &DatabaseSettings) -> PgPool {
    let pg_instance = PgPool::connect_with(configuration.without_database())
        .await
        .expect("Failed to connect to Postgres.");
    pg_instance
        .execute(format!(r#"CREATE DATABASE "{}";"#, &configuration.database_name).as_str())
        .await
        .expect("Failed to create database.");
    let test_db_pool = PgPool::connect_with(configuration.with_database())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&test_db_pool)
        .await
        .expect("Failed to migrate database.");
    test_db_pool
}
