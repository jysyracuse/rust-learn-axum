use chrono::{Duration, Utc};
use jsonwebtoken::{
    encode,
    decode,
    Header,
    EncodingKey,
    DecodingKey,
    Validation,
};
use serde::{Deserialize, Serialize};
use crate::error::AppError;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub iat: i64,
    pub exp: i64,
}

impl Claims {
    pub fn new(sub: Uuid) -> Self {
        let iat = Utc::now();
        let exp = iat + Duration::hours(24);

        Self {
            sub: sub,
            iat: iat.timestamp(),
            exp: exp.timestamp(),
        }
    }
}

pub fn sign(user_id: String) -> Result<String, String> {
    let token = encode(
        &Header::default(),
        &Claims::new(Uuid::parse_str(&user_id).unwrap()),
        &EncodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_ref()),
    )
    .map_err(|err| err.to_string())?;
    Ok(token)
}

pub fn verify(token: &str) -> Result<Claims, AppError> {
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_ref()),
        &Validation::default()
    )
    .map_err(|_| AppError::JWTTokenInvalid)?;
    Ok(token_data.claims)
}