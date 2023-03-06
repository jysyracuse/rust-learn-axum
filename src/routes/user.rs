use axum::{
  extract::{Json, Path, Query},
  http::{StatusCode, HeaderMap},
  response::{IntoResponse, Response},
  routing::{get, post, delete},
  middleware::{self},
  Extension,
  Router,
};
use axum_extra::extract::cookie::{CookieJar};
use serde_json::{json, Value};
use serde::{Serialize, Deserialize};
use utoipa::{IntoParams, ToSchema};
// use std::any::Any;
use crate::db::{self, user};
use crate::error::{AppError, AppResult};
use crate::middlewares::auth::auth_middleware;
use crate::utils::jwt::{Claims};

type Database = Extension<std::sync::Arc<db::PrismaClient>>;

/*

/api/users => GET, POST
/api/users/:id => GET
/api/users/:id/update_password => POST

*/
#[derive(Deserialize, IntoParams)]
pub struct Pagination {
  page: u8,
  page_size: u8,
}

impl Default for Pagination {
    fn default() -> Self {
        Self { page: 1, page_size: 30 }
    }
}

#[derive(Deserialize, IntoParams)]
pub struct Status {
  status: Option<String>,

}

impl Default for Status {
  fn default() -> Self {
      Self { status: Some("all".to_string()) }
  }
}

#[derive(Serialize)]
pub struct UsersResponseData {
  list: Vec<user::Data>,
  count: i64,
}

#[derive(Serialize, ToSchema)]
pub struct UsersResponse {
    code: String,
    message: String,
    data: UsersResponseData,
}

pub fn create_route() -> Router {
  Router::new()
      .route("/users", get(get_users_api))
      .route_layer(middleware::from_fn(auth_middleware))
}

#[utoipa::path(
  get,
  path = "/users",
  responses(
      (status = 200, description = "Users found successfully", body=UsersResponse),
      (status = UNAUTHORIZED, description = "Not Logged In")
  ),
  params(
    Pagination,
    Status,
  )
)]
pub async fn get_users_api(
  Extension(claims): Extension<Claims>,
  db: Database,
  Query(pagination): Query<Pagination>,
  Query(status): Query<Status>,
) -> AppResult<Json<UsersResponse>> {
  println!("Logged User from Auth Middleware: {}", claims.sub.to_string());

  let mut users_filter = vec![];

  let user_objs = db
      .user()
      .find_many(users_filter.clone())
      // .select(db::user::select!({ id }))
      .exec()
      .await
      .unwrap();

  let user_count = db
      .user()
      .count(users_filter.clone())
      .exec()
      .await?;
  
  let res_json_data = UsersResponseData {
    list: user_objs,
    count: user_count,
  };

  let res_json = UsersResponse {
    code: "200".to_string(),
    message: "OK".to_string(),
    data: res_json_data,
  };

  Ok(Json(res_json))
}
