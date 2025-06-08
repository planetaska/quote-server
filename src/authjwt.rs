//! JWT authentication module for the Quotes Server.
//!
//! Provides JWT token generation, validation, and user registration functionality.
//!
use axum::{
    Json,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// JWT signing and verification keys
#[derive(Clone)]
pub struct JwtKeys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl JwtKeys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Claims {
    pub iss: String, // issuer
    pub sub: String, // subject (user identifier)
    pub exp: u64,    // expiration time
}

/// User registration request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Registration {
    pub full_name: String,
    pub email: String,
    pub password: String,
}

/// Authentication response body containing JWT token
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuthBody {
    pub access_token: String,
    pub token_type: String,
}

impl AuthBody {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

/// Authentication errors
#[derive(Debug)]
pub enum AuthError {
    TokenCreation,
    InvalidToken,
    WrongCredentials,
    MissingCredentials,
    TokenExpired,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::TokenCreation => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Token creation failed")
            }
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token expired"),
        };
        let body = Json(serde_json::json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

/// Read secret from file
pub async fn read_secret(
    env_var: &str,
    default: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let secretf = std::env::var(env_var).unwrap_or_else(|_| default.to_owned());
    let secret = tokio::fs::read_to_string(secretf).await?;
    Ok(secret.trim().to_string())
}

/// Generate JWT keys from secret
pub async fn make_jwt_keys() -> Result<JwtKeys, Box<dyn std::error::Error>> {
    let secret = read_secret("JWT_SECRET", "./credentials.txt").await?;
    Ok(JwtKeys::new(secret.as_bytes()))
}

/// Generate JWT token for user registration
pub fn make_jwt_token(
    jwt_keys: &JwtKeys,
    reg_key: &str,
    registration: &Registration,
) -> Result<AuthBody, AuthError> {
    if registration.password != reg_key {
        return Err(AuthError::WrongCredentials);
    }

    let iss = "quote-server.localhost".to_string();
    let sub = format!("{} <{}>", registration.full_name, registration.email);
    let exp = (Utc::now() + TimeDelta::days(1)).timestamp();
    let exp = u64::try_from(exp).unwrap();

    let claims = Claims { iss, sub, exp };
    let header = Header::new(Algorithm::HS512);
    let token =
        encode(&header, &claims, &jwt_keys.encoding).map_err(|_| AuthError::TokenCreation)?;

    Ok(AuthBody::new(token))
}

/// Validate JWT token and extract claims
pub fn validate_token(jwt_keys: &JwtKeys, token: &str) -> Result<Claims, AuthError> {
    let validation = Validation::new(Algorithm::HS512);

    match decode::<Claims>(token, &jwt_keys.decoding, &validation) {
        Ok(token_data) => {
            let now = Utc::now().timestamp() as u64;
            if token_data.claims.exp < now {
                Err(AuthError::TokenExpired)
            } else {
                Ok(token_data.claims)
            }
        }
        Err(_) => Err(AuthError::InvalidToken),
    }
}

/// Axum extractor for JWT authentication
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let authorization = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok())
            .ok_or(AuthError::MissingCredentials)?;

        // Extract Bearer token
        let token = authorization
            .strip_prefix("Bearer ")
            .ok_or(AuthError::InvalidToken)?;

        // Get JWT keys from app state
        let jwt_keys = parts
            .extensions
            .get::<JwtKeys>()
            .ok_or(AuthError::InvalidToken)?;

        // Validate token and return claims
        validate_token(jwt_keys, token)
    }
}
