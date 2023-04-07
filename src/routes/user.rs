use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// Get all users via GET
#[get("/")]
#[tracing::instrument(name = "Get All Users", skip(db_pool))]
async fn get_users(db_pool: web::Data<PgPool>) -> HttpResponse {
    let result = get_all_users_repository(&db_pool).await;
    match result {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

#[tracing::instrument(name = "Get All Users In Database", skip(db_pool))]
async fn get_all_users_repository(db_pool: &PgPool) -> Result<Vec<GetUser>, sqlx::Error> {
    let users = sqlx::query_as!(
        GetUser,
        r#"
        SELECT * FROM users
        "#
    )
    .fetch_all(db_pool)
    .await?;
    Ok(users)
}

#[tracing::instrument(name = "Get User", skip(db_pool),fields(id = %id))]
// Get a user by id via GET
#[get("/{id}")]
async fn get_user(id: web::Path<String>, db_pool: web::Data<PgPool>) -> HttpResponse {
    // Parse id to uuid
    let id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid id"),
    };
    // Get user from database
    let result = get_user_by_id_repository(id, &db_pool).await;
    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

// Create a new user via POST
#[derive(Debug, Serialize, Deserialize)]
struct GetUser {
    id: Uuid,
    name: String,
    email: String,
    password: String,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}
#[tracing::instrument(name = "Get User In Database", skip(id,db_pool),fields(id = %id))]
async fn get_user_by_id_repository(id: Uuid, db_pool: &PgPool) -> Result<GetUser, sqlx::Error> {
    let user = sqlx::query_as!(
        GetUser,
        r#"
        SELECT * FROM users WHERE id = $1
        "#,
        id
    )
    .fetch_one(db_pool)
    .await?;
    Ok(user)
}

// Create a new user via POST
#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct FormUser {
    name: String,
    email: String,
    password: String,
}
#[tracing::instrument(name = "Create User", skip(form,db_pool),fields(name = %form.name, email = %form.email))]
#[post("/")]
async fn create_user(
    _req: HttpRequest,
    form: web::Form<FormUser>,
    db_pool: web::Data<PgPool>,
) -> HttpResponse {
    let result = create_user_repository(form.into_inner(), &db_pool).await;
    match result {
        Ok(_) => HttpResponse::Ok().body("User created"),
        Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

#[tracing::instrument(name = "Create User In Database", skip(user,db_pool),fields(name = %user.name, email = %user.email))]
async fn create_user_repository(user: FormUser, db_pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO users (id, name, email, password)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        user.name,
        user.email,
        user.password
    )
    .execute(db_pool)
    .await?;
    Ok(())
}

// Update a user via PUT
#[derive(serde::Deserialize)]
struct UpdateUser {
    name: String,
    email: String,
}
#[tracing::instrument(name = "Update User", skip(form,db_pool),fields(name = %form.name, email = %form.email))]
#[put("/{id}")]
async fn update_user(
    _req: HttpRequest,
    id: web::Path<i32>,
    form: web::Form<UpdateUser>,
    db_pool: web::Data<PgPool>,
) -> HttpResponse {
    HttpResponse::Ok().body(format!("User: {} {} {}", id, form.name, form.email))
}

// Delete a user via DELETE
#[delete("/{id}")]
#[tracing::instrument(name = "Delete User", skip(id,db_pool),fields(id = %id))]
async fn delete_user(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> HttpResponse {
    HttpResponse::Ok().body(format!("User: {}", id))
}

pub fn init_user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .service(get_users)
            .service(get_user)
            .service(create_user)
            .service(update_user)
            .service(delete_user),
    );
}
