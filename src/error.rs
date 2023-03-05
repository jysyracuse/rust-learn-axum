use axum::{
  http::{header, StatusCode},
  Json,
  response::{IntoResponse, Response},
};

use serde_json::{json, Value};
use thiserror::Error;

use prisma_client_rust::{
  prisma_errors::query_engine::{RecordNotFound, UniqueKeyViolation},
  QueryError,
};


#[derive(Error, Debug)]
pub enum AppError {
    // #[error("invalid credentials")]
    // InvalidCredentialsError,
    // #[error("user exists")]
    // UserExistsError(String),
    // #[error("invalid jwt token")]
    // InvalidJWTTokenError,
    // #[error("jwt token creation error")]
    // JWTTokenCreationError,
    // #[error("authorization header required")]
    // AuthHeaderRequiredError,
    // #[error("invalid auth header")]
    // InvalidAuthHeaderError,
    // #[error("not authorized")]
    // NotAuthorizedError,
    #[error("DB Error")]
    PrismaError(QueryError),
    #[error("Record Not Found")]
    RecordNotFound,
    #[error("Record Existed")]
    RecordExisted,
    #[error("Credentials Error")]
    WrongCredentials,
    #[error("Invalid Token")]
    JWTTokenInvalid,
}

// pub enum AppError {
//   PrismaError(QueryError),
//   // JWTError(ErrorKind),
//   NotFound,
//   WrongCredentials,
// }

pub type AppResult<T> = Result<T, AppError>;
// pub type AppJsonResult<T> = AppResult<Json<T>>

impl From<QueryError> for AppError {
  fn from(error: QueryError) -> Self {
      match error {
        e if e.is_prisma_error::<RecordNotFound>() => AppError::RecordNotFound,
        e => AppError::PrismaError(e),
      }
  }
}

// impl From<ErrorKind> for AppError {
//   fn from(error: ErrorKind) -> Self {
//     AppError::JWTError(error)
//   }
// }

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
      let (status, error_message) = match self {
          AppError::PrismaError(error) if error.is_prisma_error::<UniqueKeyViolation>() => {
            (StatusCode::CONFLICT, "记录已存在")
          }
          AppError::PrismaError(_) => {
            (StatusCode::BAD_REQUEST, "查询出错")
          }
          AppError::RecordNotFound => {
            (StatusCode::NOT_FOUND, "未找到结果")
          }
          AppError::RecordExisted => {
            (StatusCode::BAD_REQUEST, "已有该记录")
          }
          AppError::WrongCredentials => {
            (StatusCode::UNAUTHORIZED, "错误的用户名/密码")
          }
          AppError::JWTTokenInvalid => {
            (StatusCode::UNAUTHORIZED, "登陆状态错误")
          }
      };

      let body = Json(json!({
          "code": status.to_string(),
          "message": error_message,
      }));

      (status, body).into_response()
  }
}