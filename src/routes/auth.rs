use axum::{
  extract::Json,
  routing::post,
  Extension,
  Router,
};
use axum_extra::extract::cookie::{CookieJar, Cookie};
use serde::{Serialize, Deserialize};
use bcrypt::{DEFAULT_COST, verify, hash};
use crate::db::{self, user};
use crate::error::{AppError, AppResult};
use crate::utils::jwt::sign;
type Database = Extension<std::sync::Arc<db::PrismaClient>>;

/*

Plan for Auth API

/login => POST
/register => POST

*/
pub fn create_route() -> Router {
  Router::new()
      .route("/login", post(login_api))
      .route("/register", post(register_api))
}

/// Define Login Schemas
#[derive(Deserialize)]
pub struct LoginRequestBody {
    name: String,
    password: String,
}

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
  let user_obj_q: Option<user::Data> = db
      .user()
      .find_first(vec![user::name::equals(input.name.clone())])
      .exec()
      .await
      .unwrap();
      
  if user_obj_q.is_some() == false {
      /// Throw Error when user not found
      return Err(AppError::WrongCredentials)
  }

  let user_obj = user_obj_q.unwrap();


  verify(&input.password, &user_obj.password).map_err(|_| AppError::WrongCredentials)?;

  // set jwt cookie
  let jwt_data = sign(user_obj.id.to_string()).unwrap();

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

/// Define Register Schemas
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
      (status = 400, description = "Record Not Existed"),
      (status = 400, description = "Password Not Match")
  ),
)]
async fn register_api(
  db: Database,
  Json(input): Json<RegisterRequestBody>,
) -> AppResult<Json<RegisterResponse>> {
    /// Verify Passwords are same
    if !&input.password.eq(&input.password_confirm) {
      return Err(AppError::PasswordDontMatch)
    }

    /// Existed User Check
    let existed_user_obj = db
        .user()
        .find_first(vec![user::name::equals(input.name.clone())])
        .exec()
        .await
        .unwrap();
    
    /// Don't allow if there is already record with user name
    if existed_user_obj.is_some() == true {
        return Err(AppError::RecordExisted)
    } else {
        let password_hash = hash(input.password, DEFAULT_COST).unwrap();
        // traccing::info!("password_hash: {}", &password_hash);

        let user_obj = db
            .user()
            .create(input.name, password_hash, vec![])
            .exec()
            .await
            .unwrap();

        /// Response
        let res_json = RegisterResponse {
          code: "200".to_string(),
          message: "OK".to_string(),
          data: user_obj.id.to_string(),
        };

        return Ok(Json(res_json))
    }
}