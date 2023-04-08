use actix_template::routes::GetUser;
use reqwest::{self, Client};
use std::collections::HashMap;
use uuid::Uuid;

mod common;

#[actix_web::test]
#[serial_test::serial]
async fn user_crud() -> Result<(), Box<dyn std::error::Error>> {
    // Spawn App
    let app = common::spawn_app().await;
    // Create Client
    let client = Client::new();

    // User
    let mut user_map = HashMap::new();
    user_map.insert("name", "test");
    user_map.insert("email", "test@gmail.com");
    user_map.insert("password", "test");

    // Client Act on Server
    // the health check is exposed at /health_check;
    // the health check is behind a GET method;
    let response = client
        .post(&format!("{}/user/", &app.address))
        .json(&user_map)
        .send()
        .await
        .expect("Failed to execute request.");

    // Compare response
    // the health check always returns a 200;
    // the health check’s response has no body;
    assert_eq!(response.status().as_u16(), 200);
    let id = match response.json::<Uuid>().await {
        Ok(id) => id,
        Err(_) => panic!("Failed to get response id"),
    };

    // Check if user exist in database
    let user = sqlx::query!(r#"SELECT * FROM users WHERE id = $1"#, &id)
        .fetch_one(&app.db_pool)
        .await?;
    assert_eq!(user.name, "test");
    assert_eq!(user.email, "test@gmail.com");

    // Try to create user with same email
    // Client Act on Server
    let response = client
        .post(&format!("{}/user/", &app.address))
        .json(&user_map)
        .send()
        .await
        .expect("Failed to execute request.");
    println!("{:?}", response);
    // Compare response
    assert_eq!(response.status().as_u16(), 400);

    // Get user by id
    let response = client
        .get(&format!("{}/user/{}", &app.address, &id))
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(response.status().as_u16(), 200);
    let user = match response.json::<GetUser>().await {
        Ok(user) => user,
        Err(_) => panic!("Failed to get response id"),
    };
    assert_eq!(user.name, "test");
    assert_eq!(user.email, "test@gmail.com");

    // Get all users
    let response = client
        .get(&format!("{}/user/", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(response.status().as_u16(), 200);
    let users = match response.json::<Vec<GetUser>>().await {
        Ok(users) => users,
        Err(_) => panic!("Failed to get response id"),
    };
    assert!(!users.is_empty());
    assert!(users.iter().any(|user| user.id == id));

    // Update user
    let mut user_map = HashMap::new();
    user_map.insert("name", "test2");
    user_map.insert("password", "test2");
    let response = client
        .put(&format!("{}/user/{}", &app.address, &id))
        .json(&user_map)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(response.status().as_u16(), 200);

    // Check in database
    let user = sqlx::query!(r#"SELECT * FROM users WHERE id = $1"#, &id)
        .fetch_one(&app.db_pool)
        .await?;
    assert_eq!(user.name, "test2");
    assert_eq!(user.password, "test2");

    // Delete user
    let response = client
        .delete(&format!("{}/user/{}", &app.address, &id))
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(response.status().as_u16(), 200);

    // Check in database
    let user = sqlx::query!(r#"SELECT * FROM users WHERE id = $1"#, &id)
        .fetch_optional(&app.db_pool)
        .await?;
    assert!(user.is_none());

    Ok(())
}
