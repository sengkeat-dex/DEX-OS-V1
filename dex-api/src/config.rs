//! Configuration loading for the DEX API service.
//!
//! Centralizes environment parsing and keeps sensitive values wrapped in
//! secrecy primitives.

use dotenvy::dotenv;
use secrecy::SecretString;
use std::{env, num::ParseIntError};
use thiserror::Error;

/// Runtime configuration for the API service.
#[derive(Clone)]
pub struct Config {
    pub database_url: SecretString,
    pub jwt_secret: SecretString,
    pub server_port: u16,
}

impl Config {
    /// Load configuration from environment variables, honoring values supplied
    /// via a `.env` file when present.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").map_err(|_| ConfigError::Missing("DATABASE_URL"))?;
        let jwt_secret = env::var("JWT_SECRET").map_err(|_| ConfigError::Missing("JWT_SECRET"))?;

        let server_port = parse_server_port(env::var("SERVER_PORT").ok())?;

        Ok(Self {
            database_url: SecretString::from(database_url),
            jwt_secret: SecretString::from(jwt_secret),
            server_port,
        })
    }
}

fn parse_server_port(raw: Option<String>) -> Result<u16, ConfigError> {
    match raw {
        Some(value) => value
            .parse::<u16>()
            .map_err(|err| ConfigError::InvalidPort { value, err }),
        None => Ok(3030),
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required environment variable {0}")]
    Missing(&'static str),
    #[error("invalid SERVER_PORT value {value}: {err}")]
    InvalidPort { value: String, err: ParseIntError },
}
