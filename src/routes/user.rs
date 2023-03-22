use axum::{
  extract::{Json, Path, Query},
  routing::{get, post, delete},
  middleware::{self},
  Extension,
  Router,
};
use bcrypt::{DEFAULT_COST, hash};
use serde::{Serialize, Deserialize};
use utoipa::{IntoParams, ToSchema};
use serde_json::json;
use crate::db::{self, user};
use crate::error::{AppError, AppResult};
use crate::middlewares::auth::auth_middleware;
use crate::utils::jwt::{Claims};
type Database = Extension<std::sync::Arc<db::PrismaClient>>;

/*

Plan for User API

/api/users => GET, POST
/api/users/:user_id => GET
/api/users/:user_id/update_password => POST
/api/users/:user_id => DELETE

*/
#[derive(Deserialize, IntoParams)]
pub struct GetUsersAPIQuery {
    page: Option<i32>,
    page_size: Option<i32>,
    status: Option<i32>,
}

#[derive(Serialize)]
struct UsersData {
    list: Vec<user::Data>,
    count: i64,
}

#[derive(Serialize)]
pub struct GetUsersAPIResponse {
    code: String,
    message: String,
    data: UsersData,
}

pub fn create_route() -> Router {
  Router::new()
      .route("/users", get(get_users_api))
      .route("/users/:user_id", get(get_user_api))
      .route("/users/:user_id/update_password", post(update_user_password_api))
      .route("/users/:user_id", delete(delete_user_api))
      .layer(middleware::from_fn(auth_middleware))
}

#[utoipa::path(
  get,
  path = "/users",
  responses(
      (status = 200, description = "Users found successfully"),
      (status = UNAUTHORIZED, description = "Not Logged In")
  ),
  params(
    GetUsersAPIQuery,
  )
)]
pub async fn get_users_api(
  Extension(_claims): Extension<Claims>,
  db: Database,
  Query(query): Query<GetUsersAPIQuery>,
) -> AppResult<Json<GetUsersAPIResponse>> {
  let mut users_filter = vec![];

  // apply filter
  

  let status_ind = query.status.unwrap_or(99);
  tracing::info!("{}", status_ind);
  if status_ind < 99 {
    users_filter.push(user::status::equals(status_ind))
  }

  let user_objs = db
      .user()
      .find_many(users_filter.clone())
      // .select()
      // apply pagination
      .take(i64::from(query.page_size.unwrap_or(10)))
      .skip(i64::from(query.page_size.unwrap_or(10) * (query.page.unwrap_or(1) - 1)))
      .exec()
      .await
      .unwrap();

  let user_count = db
      .user()
      .count(users_filter.clone())
      .exec()
      .await?;
  
  let res_json_data = UsersData {
    list: user_objs,
    count: user_count,
  };

  let res_json = GetUsersAPIResponse {
    code: "200".to_string(),
    message: "OK".to_string(),
    data: res_json_data,
  };

  Ok(Json(res_json))
}

#[derive(Deserialize, IntoParams)]
pub struct GetUserParams {
  user_id: String,
}

#[derive(Serialize)]
pub struct GetUserAPIResponse {
    code: String,
    message: String,
    data: user::Data,
}

#[utoipa::path(
  get,
  path = "/users/:user_id",
  responses(
      (status = 200, description = "User found successfully"),
      (status = UNAUTHORIZED, description = "Not Logged In")
  ),
  params(
    GetUserParams,
  )
)]
pub async fn get_user_api(
  Extension(_claims): Extension<Claims>,
  db: Database,
  Path(GetUserParams{user_id}): Path<GetUserParams>,
) -> AppResult<Json<GetUserAPIResponse>> {
  let user_obj_q = db
      .user()
      .find_unique(user::id::equals(user_id))
      .exec()
      .await?;
      // .unwrap()
      // .unwrap();


      // .map_err(|_| AppError::RecordNotFound)?
      // .unwrap();
  tracing::info!("{}", json!(user_obj_q));
  if user_obj_q.is_none() {
    return Err(AppError::RecordNotFound)
  }


  let res_json = GetUserAPIResponse {
    code: "200".to_string(),
    message: "OK".to_string(),
    data: user_obj_q.unwrap(),
  };

  Ok(Json(res_json))
}

#[derive(Deserialize, IntoParams)]
pub struct UpdateUserPasswordParams {
  user_id: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateUserPasswordBody {
    password: String,
    password_confirm: String,
}

#[derive(Serialize)]
pub struct UpdateUserPasswordResponse {
    code: String,
    message: String,
    data: String,
}

#[utoipa::path(
  post,
  path = "/users/:user_id/update_password",
  request_body = UpdateUserPasswordBody,
  responses(
      (status = 200, description = "Password updated successfully"),
      (status = UNAUTHORIZED, description = "Not Logged In"),
      (status = BAD_REQUEST, description = "Password Dont Match")
  ),
  params(
    UpdateUserPasswordParams,
  )
)]
pub async fn update_user_password_api(
    Extension(claims): Extension<Claims>,
    db: Database,
    Path(UpdateUserPasswordParams{user_id}): Path<UpdateUserPasswordParams>,
    Json(input): Json<UpdateUserPasswordBody>,
) -> AppResult<Json<UpdateUserPasswordResponse>> {
    // Avoid user delete his/her self
    if claims.sub.to_string().eq(&user_id) {
      return Err(AppError::OperationConflict)
    }

    if !&input.password.eq(&input.password_confirm) {
      return Err(AppError::PasswordDontMatch)
    }
    let password_hash = hash(&input.password, DEFAULT_COST).unwrap();

    let user_obj = db
        .user()
        .update(
            user::id::equals(user_id),
            vec![
                user::password::set(password_hash),
            ],
        )
        .exec()
        .await
        .unwrap();

    let res_json = UpdateUserPasswordResponse {
      code: "200".to_string(),
      message: "OK".to_string(),
      data: user_obj.id.to_string(),
    };

    Ok(Json(res_json))
}


#[derive(Deserialize, IntoParams)]
pub struct DeleteUserParams {
  user_id: String,
}

#[derive(Serialize)]
pub struct DeleteUserResponse {
    code: String,
    message: String,
    data: String,
}

#[utoipa::path(
  delete,
  path = "/users/:user_id",
  responses(
      (status = 200, description = "User Delete successfully"),
      (status = BAD_REQUEST, description = "Record Not Found"),
      (status = UNAUTHORIZED, description = "Not Logged In"),
  ),
  params(
    UpdateUserPasswordParams,
  )
)]
pub async fn delete_user_api(
    Extension(claims): Extension<Claims>,
    db: Database,
    Path(DeleteUserParams{user_id}): Path<DeleteUserParams>,
) -> AppResult<Json<DeleteUserResponse>> {
    // Avoid user delete his/her self
    if claims.sub.to_string().eq(&user_id) {
      return Err(AppError::OperationConflict)
    }

    let user_obj_q = db
      .user()
      .find_unique(user::id::equals(String::from(&user_id)))
      .exec()
      .await?;

    if user_obj_q.is_none() {
      return Err(AppError::RecordNotFound)
    }

    db.user()
      .delete(user::id::equals(String::from(&user_id)))
      .exec()
      .await?;

    let res_json = DeleteUserResponse {
      code: "200".to_string(),
      message: "User Deleted".to_string(),
      data: String::from(&user_id),
    };

    Ok(Json(res_json))
}