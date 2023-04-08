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
        User,
        r#"
        SELECT * FROM users
        "#
    )
    .fetch_all(db_pool)
    .await?;
    Ok(users.into_iter().map(|user| user.into()).collect())
}

#[tracing::instrument(name = "Get User", skip(db_pool),fields(id = %id))]
// Get a user by id via GET
#[get("/{id}")]
async fn get_user(id: web::Path<Uuid>, db_pool: web::Data<PgPool>) -> HttpResponse {
    // Get user from database
    let result = get_user_by_id_repository(*id, &db_pool).await;
    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUser {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

impl From<User> for GetUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user.name,
            email: user.email,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

// Create a new user via POST
#[derive(Debug, Serialize, Deserialize)]
struct User {
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
        User,
        r#"
        SELECT * FROM users WHERE id = $1
        "#,
        id
    )
    .fetch_one(db_pool)
    .await?;
    Ok(user.into())
}

// Create a new user via POST
#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct CreateUser {
    name: String,
    email: String,
    password: String,
}
#[tracing::instrument(name = "Create User", skip(json,db_pool),fields(name = %json.name, email = %json.email))]
#[post("/")]
async fn create_user(
    _req: HttpRequest,
    json: web::Json<CreateUser>,
    db_pool: web::Data<PgPool>,
) -> HttpResponse {
    let result = create_user_repository(json.into_inner(), &db_pool).await;
    match result {
        Ok(id) => HttpResponse::Ok().json(id),
        // Found user
        Err(sqlx::Error::RowNotFound) => HttpResponse::BadRequest().body("User already exists"),
        Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

#[tracing::instrument(name = "Create User In Database", skip(user,db_pool),fields(name = %user.name, email = %user.email))]
async fn create_user_repository(user: CreateUser, db_pool: &PgPool) -> Result<Uuid, sqlx::Error> {
    // Check if user already exists
    let is_exist = check_if_user_exists(&user.email, db_pool).await;
    let is_exist = match is_exist {
        Ok(is_exist) => is_exist,
        Err(_) => return Err(sqlx::Error::RowNotFound),
    };
    if is_exist {
        return Err(sqlx::Error::RowNotFound);
    }

    let id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO users (id, name, email, password)
        VALUES ($1, $2, $3, $4)
        "#,
        &id,
        user.name,
        user.email,
        user.password
    )
    .execute(db_pool)
    .await?;
    tracing::error!("{}", &id.to_string());
    Ok(id)
}

// Update a user via PUT
#[derive(serde::Deserialize)]
pub struct UpdateUser {
    name: Option<String>,
    password: Option<String>,
}
#[tracing::instrument(name = "Update User", skip(json, db_pool) ,fields(id = %id))]
#[put("/{id}")]
async fn update_user(
    _req: HttpRequest,
    id: web::Path<String>,
    json: web::Json<UpdateUser>,
    db_pool: web::Data<PgPool>,
) -> HttpResponse {
    // Parse id to uuid
    let id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid id"),
    };
    // Update user in database
    let result = update_user_repository(id, json.into_inner(), &db_pool).await;
    match result {
        Ok(_) => HttpResponse::Ok().body("User updated"),
        // Could not found user
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("User not found"),
        Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}
#[tracing::instrument(name = "Update User In Database", skip(id, user, db_pool),fields(id = %id))]
async fn update_user_repository(
    id: Uuid,
    user: UpdateUser,
    db_pool: &PgPool,
) -> Result<(), sqlx::Error> {
    // Get user from database
    let found_user = sqlx::query_as!(
        User,
        r#"
        SELECT * FROM users WHERE id = $1
        "#,
        id
    )
    .fetch_one(db_pool)
    .await;
    let found_user = match found_user {
        Ok(user) => user,
        Err(_) => return Err(sqlx::Error::RowNotFound),
    };
    // Update user
    sqlx::query!(
        r#"
        UPDATE users SET name = $1, password = $2 WHERE id = $3
        "#,
        user.name.to_owned().unwrap_or(found_user.name),
        user.password.to_owned().unwrap_or(found_user.password),
        id
    )
    .execute(db_pool)
    .await?;
    Ok(())
}

// Delete a user via DELETE
#[delete("/{id}")]
#[tracing::instrument(name = "Delete User", skip(id,db_pool),fields(id = %id))]
async fn delete_user(id: web::Path<Uuid>, db_pool: web::Data<PgPool>) -> HttpResponse {
    // Delete user from database
    let result = delete_user_repository(*id, &db_pool).await;
    match result {
        Ok(_) => HttpResponse::Ok().body("User deleted"),
        Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

#[tracing::instrument(name = "Delete User In Database", skip(id,db_pool),fields(id = %id))]
async fn delete_user_repository(id: Uuid, db_pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM users WHERE id = $1
        "#,
        id
    )
    .execute(db_pool)
    .await?;
    Ok(())
}

async fn check_if_user_exists(email: &str, db_pool: &PgPool) -> Result<bool, sqlx::Error> {
    let found_user = sqlx::query!(
        r#"
        SELECT * FROM users WHERE email = $1
        "#,
        email
    )
    .fetch_optional(db_pool)
    .await?;
    if found_user.is_some() {
        return Ok(true);
    }
    Ok(false)
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
