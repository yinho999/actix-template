use std::net::TcpListener;
use sqlx::{Executor, PgPool};
use uuid::Uuid;
use actix_template::{DatabaseSettings, get_configuration};
use actix_template::telemetry::init_subscriber;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    init_subscriber("test".into(), "debug".into());
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let address = format!("http://{}", listener.local_addr().unwrap());
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let db_pool = configure_test_database(&configuration.database).await;
    let server = actix_template::run(listener, db_pool.clone()).expect("Failed to bind address");
    let _ = actix::spawn(server);
    TestApp {
        address,
        db_pool,
    }
}

async fn configure_test_database(configuration: &DatabaseSettings) -> PgPool {
    let mut PgInstance = PgPool::connect_with(configuration.without_database())
        .await
        .expect("Failed to connect to Postgres.");
    PgInstance.execute(format!(r#"CREATE DATABASE "{}";"#, &configuration.database_name).as_str())
        .await
        .expect("Failed to create database.");
    let mut test_db_pool = PgPool::connect_with(configuration.with_database())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations").run(&test_db_pool).await.expect("Failed to migrate database.");
    test_db_pool
}

#[actix_web::test]
async fn health_check_works() {
    // Spawn App
    let app = spawn_app().await;
    // Create Client
    let client = reqwest::Client::new();
    // Client Act on Server
    // the health check is exposed at /health_check;
    // the health check is behind a GET method;
    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Compare response
    // the health check always returns a 200;
    // the health check’s response has no body;
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}