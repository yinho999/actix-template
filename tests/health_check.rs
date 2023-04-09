mod common;

#[actix_web::test]
async fn health_check_works() {
    // Spawn App
    let app = common::spawn_app().await;
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
