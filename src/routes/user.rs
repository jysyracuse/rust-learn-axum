use axum::{
  extract::{Json, Path, Query},
  http::{StatusCode, HeaderMap},
  response::{IntoResponse, Response},
  routing::{get, post, delete},
  middleware::{self},
  Extension,
  Router,
};
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
// use prisma_client_rust::{
//   prisma_errors::query_engine::{RecordNotFound, UniqueKeyViolation},
//   QueryError,
// };
use axum_extra::extract::cookie::{CookieJar};
use serde_json::{json, Value};
use serde::Deserialize;
// use std::any::Any;
use crate::db::{self, user};
use crate::error::{AppError, AppResult};
use crate::middlewares::auth::auth_middleware;
use crate::utils::jwt::{Claims};

type Database = Extension<std::sync::Arc<db::PrismaClient>>;
// type AppResult<T> = Result<T, AppError>;
// type AppJsonResult<T> = AppResult<Json<T>>;

// enum AppError {
//   PrismaError(QueryError),
//   NotFound,
// }

// Define all your requests schema
#[derive(Deserialize)]
struct Pagination {
  page: u8,
  page_size: u8,
}

impl Default for Pagination {
    fn default() -> Self {
        Self { page: 1, page_size: 30 }
    }
}

#[derive(Deserialize)]
struct Status {
  status: Option<String>,

}
/*

/api/user => GET, POST
/api/user/:name => PUT, DELETE
/comment => POST

*/
pub fn create_route() -> Router {
  Router::new()
      .route("/users", get(get_users_api))
      .route_layer(middleware::from_fn(auth_middleware))
}

async fn get_users_api(
  Extension(claims): Extension<Claims>,
  db: Database,
  Query(pagination): Query<Pagination>,
  Query(status): Query<Status>,
) -> AppResult<Json<Value>> {
  println!("Logged User from Auth Middleware: {}", claims.sub.to_string());

  let mut res = json!({
    "code": 200,
    "message": String::from("success"),
    "data": null,
  });

  let users = db
      .user()
      .find_many(vec![])
      // .select(db::user::select!({ id }))
      .exec()
      .await
      .unwrap();

  *res.get_mut("data").unwrap() = json!(users);
  Ok(Json(res))
}
