use axum::{
  extract::Json,
  routing::post,
  Extension,
  Router,
};
use axum_extra::extract::cookie::{CookieJar, Cookie};
use serde::{Serialize, Deserialize};
use std::any::Any;
use bcrypt::{DEFAULT_COST, verify, hash};
use utoipa::{ToSchema};
use crate::db::{self, user};
use crate::error::{AppError, AppResult};
use crate::utils::jwt::sign;

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
      .route("/login", post(login_api))
      .route("/register", post(register_api))
}

// Define login schema
#[derive(Deserialize)]
pub struct LoginRequestBody {
    name: String,
    password: String,
}

// Define login response
#[derive(Serialize)]
pub struct LoginResponse {
    code: String,
    message: String,
    data: user::Data,
}

#[utoipa::path(
  post,
  path = "/login",
  request_body = LoginRequestBody,
  responses(
      (status = 200, description = "Login successfully"),
      (status = 401, description = "User Not Existed"),
      (status = 401, description = "User/Password Incorrect"),
  ),
)]
async fn login_api(
  db: Database,
  cookie_jar: CookieJar,
  Json(input): Json<LoginRequestBody>,
) -> Result<(CookieJar, Json<LoginResponse>), AppError> {
  // tracing::info!("input -> username: {}, password: {}",input.name, input.password);

  let user_obj_q: Option<user::Data> = db
      .user()
      .find_first(vec![user::name::equals(input.name.clone())])
      .exec()
      .await
      .unwrap();
      
  if user_obj_q.is_some() == false {
    // If can not find user from db
    return Err(AppError::WrongCredentials)
  }

  let user_obj = user_obj_q.unwrap();
  let pass_correct = verify(&input.password, &user_obj.password).map_err(|_| AppError::WrongCredentials)?;
  // tracing::info!("pass_correct: {}", &pass_correct);
  // if !&pass_correct {
  //   return Err(AppError::WrongCredentials)
  // }
  // set jwt cookie
  let jwt_data = sign(user_obj.id.to_string()).unwrap();
  tracing::info!("jwt data: {}", jwt_data);
  let set_cookie = Cookie::build("user", jwt_data)
    .path("/")
    .http_only(true)
    .secure(false)
    .finish();
  let new_cookie_jar = cookie_jar.add(set_cookie);

  let res_json = LoginResponse {
    code: "200".to_string(),
    message: "Login Success".to_string(),
    data: user_obj
  };

  return Ok((
    new_cookie_jar,
    Json(res_json)
  ))
}

// Define login schema
#[derive(Deserialize)]
pub struct RegisterRequestBody {
    name: String,
    password: String,
    password_confirm: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    code: String,
    message: String,
    data: String,
}

#[utoipa::path(
  post,
  path = "/register",
  request_body = LoginRequestBody,
  responses(
      (status = 200, description = "Register successfully"),
      (status = 400, description = "Record Not Existed")
  ),
)]
async fn register_api(
  db: Database,
  Json(input): Json<RegisterRequestBody>,
) -> AppResult<Json<RegisterResponse>> {
  tracing::info!("username:{},password:{},password_confirm: {}",input.name, input.password, input.password_confirm);
  // let req_data = input.name.unwrap();
  if !&input.password.eq(&input.password_confirm) {
    return Err(AppError::PasswordDontMatch)
  }

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

      let res_json = RegisterResponse {
        code: "200".to_string(),
        message: "OK".to_string(),
        data: user_obj.id.to_string(),
      };

      return Ok(Json(res_json))
  }
}