use actix_web::{delete, get, post, put, web, HttpResponse};

// Get a user by id via GET
#[get("/{id}")]
async fn get_user(id: web::Path<i32>) -> HttpResponse {
    HttpResponse::Ok().body(format!("User: {}", id))
}

// Create a new user via POST
#[derive(serde::Deserialize)]
struct FormUser {
    name: String,
    email: String,
}

#[post("/")]
async fn create_user(form: web::Form<FormUser>) -> HttpResponse {
    HttpResponse::Ok().body(format!("User: {} {}", form.name, form.email))
}

// Delete a user via DELETE
#[delete("/{id}")]
async fn delete_user(id: web::Path<i32>) -> HttpResponse {
    HttpResponse::Ok().body(format!("User: {}", id))
}

// Update a user via PUT
#[derive(serde::Deserialize)]
struct UpdateUser {
    name: String,
    email: String,
}
#[put("/{id}")]
async fn update_user(id: web::Path<i32>, form: web::Form<UpdateUser>) -> HttpResponse {
    HttpResponse::Ok().body(format!("User: {} {} {}", id, form.name, form.email))
}

pub fn init_user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user);
    cfg.service(create_user);
    cfg.service(delete_user);
    cfg.service(update_user);
}
