//! JWT authentication helpers for Warp filters.

use jsonwebtoken::{decode, DecodingKey, Validation};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};
use thiserror::Error;
use warp::reject::Reject;

/// Shared authentication manager that validates bearer tokens.
#[derive(Clone)]
pub struct AuthManager {
    decoding_key: Arc<DecodingKey>,
    validation: Validation,
}

impl AuthManager {
    pub fn new(secret: &SecretString) -> Self {
        let decoding_key = DecodingKey::from_secret(secret.expose_secret().as_bytes());
        let mut validation = Validation::default();
        validation.validate_exp = true;
        Self {
            decoding_key: Arc::new(decoding_key),
            validation,
        }
    }

    pub fn verify_bearer(&self, header_value: &str) -> Result<Claims, AuthError> {
        let token = header_value
            .strip_prefix("Bearer ")
            .ok_or(AuthError::MissingBearer)?;

        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|err| AuthError::InvalidToken(err.to_string()))?;

        Ok(token_data.claims)
    }
}

/// JWT claims we expect from clients.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    #[serde(default)]
    pub aud: Option<String>,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("authorization header must use the Bearer scheme")]
    MissingBearer,
    #[error("invalid token: {0}")]
    InvalidToken(String),
}

#[derive(Debug)]
pub struct AuthRejection(pub AuthError);

impl Reject for AuthRejection {}

impl fmt::Display for AuthRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "authentication failed: {}", self.0)
    }
}
