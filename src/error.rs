use axum::{
  http::StatusCode,
  Json,
  response::{IntoResponse, Response},
};
use serde::{Serialize};
use serde_json::{json};
use thiserror::Error;

use prisma_client_rust::{
  prisma_errors::query_engine::{RecordNotFound, UniqueKeyViolation},
  QueryError,
};


#[derive(Error, Debug)]
pub enum AppError {
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
    #[error("Password Dont match")]
    PasswordDontMatch,
}

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

/// Define Error Response Struct
#[derive(Serialize)]
struct ErrorResponse {
    code: String,
    message: String,
    data: String,
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
      let (status, error_message) = match self {
          AppError::PrismaError(error) if error.is_prisma_error::<UniqueKeyViolation>() => {
            (StatusCode::CONFLICT, "Record existed")
          }
          AppError::PrismaError(_) => {
            (StatusCode::BAD_REQUEST, "Query Error")
          }
          AppError::RecordNotFound => {
            (StatusCode::NOT_FOUND, "Record not found")
          }
          AppError::RecordExisted => {
            (StatusCode::BAD_REQUEST, "Record existed")
          }
          AppError::WrongCredentials => {
            (StatusCode::UNAUTHORIZED, "Username/password incorrect")
          }
          AppError::JWTTokenInvalid => {
            (StatusCode::UNAUTHORIZED, "Login Error")
          }
          AppError::PasswordDontMatch => {
            (StatusCode::BAD_REQUEST, "Passwords don't match")
          }
      };

      let res_json = ErrorResponse {
        code: status.to_string()
        .chars()
        .filter(|c| c.is_digit(10))
        .collect(),
        message: error_message.to_string(),
        data: "".to_string()
      };

      tracing::debug!("{}", json!(&res_json));
      let body = Json(json!(res_json));

      (status, body).into_response()
  }
}