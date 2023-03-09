use axum::{
    http::Request,
    response::{IntoResponse, Response},
    middleware::Next,
};
use axum_extra::extract::cookie::{CookieJar, Cookie};
use crate::utils::jwt::verify;
use crate::error::{AppError, AppResult};

/* 
  Middleware Example:
    User can query API's with cookie (or you can use redis for session here as well)
    and store the User Id in context which can be used in request handler
*/

pub async fn auth_middleware<B>(
  cookie_jar: CookieJar,
  mut req: Request<B>,
  next: Next<B>,
) -> Result<Response, AppError> {
  if let Some(user_token) = cookie_jar.get("user") {
    // Note: Print user's cookie
    println!("User's Cookie: {}", &user_token.value());

    match verify(&user_token.value()) {
      Ok(claims) => {
        println!("Logged User's Id: {}", claims.sub);
        req.extensions_mut().insert(claims);
      }
      Err(e) => {
        println!("Cookie Validate failed");
        return Err(AppError::JWTTokenInvalid)
      }
    };
  } else {
    return Err(AppError::JWTTokenInvalid)
  }
  Ok(next.run(req).await)
}