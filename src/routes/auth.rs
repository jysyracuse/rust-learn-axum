use axum::{
  extract::{Json, Path, Query},
  http::StatusCode,
  response::{IntoResponseParts, IntoResponse},
  routing::{get, post, delete},
  Extension, Router,
};
use uuid::uuid;
use prisma_client_rust::{
  prisma_errors::query_engine::{RecordNotFound, UniqueKeyViolation},
  QueryError,
};

use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use std::any::Any;
use bcrypt::{DEFAULT_COST, hash, verify};

use axum_extra::extract::cookie::{CookieJar, Cookie};
use crate::db::{self, user};
use crate::error::{AppError, AppResult};
use crate::utils::jwt::sign;
use crate::middlewares::auth::auth_middleware;

type Database = Extension<std::sync::Arc<db::PrismaClient>>;



/*

/api/users => GET, POST
/api/users/:id => GET, PUT

*/
fn print_type<T: Any>(value: &T) {
  println!("Type of value: {:?}", std::any::type_name::<T>());
}

pub fn create_route() -> Router {
  Router::new()
      .route("/login", post(login_handler))
      .route("/register", post(register_handler))
}

// Define login schema
#[derive(Deserialize)]
struct LoginRequest {
    name: String,
    password: String,
}

// Define login response
#[derive(Serialize)]
struct LoginResponse {
    code: u8,
    message: String,
    data: user::Data,
}

async fn login_handler(
  db: Database,
  cookie_jar: CookieJar,
  Json(input): Json<LoginRequest>,
) -> Result<(CookieJar, Json<Value>), AppError> {
  tracing::info!("input -> username: {}, password: {}",input.name, input.password);

  let user_obj: Option<user::Data> = db
      .user()
      .find_first(vec![user::name::equals(input.name.clone())])
      .exec()
      .await
      .unwrap();
  if user_obj.is_some() == false {
    // If can not find user from db
    return Err(AppError::WrongCredentials)
  }
  
  let jwt_data = sign(user_obj.as_ref().unwrap().id.to_string()).unwrap();
    tracing::info!("jwt data: {}", jwt_data);
    let set_cookie = Cookie::build("user", jwt_data)
      .path("/")
      .http_only(true)
      .secure(false)
      .finish();

    let new_cookie_jar = cookie_jar.add(set_cookie);

    Ok((new_cookie_jar,
      Json(json!({
        "code": 200,
        "message": "Login Success",
        "data": {
          "id": user_obj.as_ref().unwrap().id,
          "name": user_obj.as_ref().unwrap().name,
        }
      }))))
}

// Define login schema
#[derive(Deserialize)]
struct RegisterRequest {
    name: String,
    password: String,
}

async fn register_handler(
  db: Database,
  Json(input): Json<LoginRequest>,
) -> AppResult<Json<Value>> {
  println!("username:{},password:{}",input.name, input.password);
  // let req_data = input.name.unwrap();

  let existed_user_obj = db
      .user()
      .find_first(vec![user::name::equals(input.name.clone())])
      .exec()
      .await
      .unwrap();

  if existed_user_obj.is_some() == true {
      return Err(AppError::RecordExisted)
  } else {
      let password_hash = hash(input.password, DEFAULT_COST).unwrap();
      println!("password_hash: {}", &password_hash);
      let user_obj = db
          .user()
          .create(input.name, password_hash.to_string(), vec![])
          .exec()
          .await
          .unwrap();

      return Ok(Json(json!({
        "code": 200,
        "message": "Register Success",
        "data": ""
      })))
  }
}