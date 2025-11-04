//! Configuration loading for the DEX API service.
//!
//! Centralizes environment parsing and keeps sensitive values wrapped in
//! secrecy primitives.

use dotenvy::dotenv;
use secrecy::SecretString;
use std::{collections::HashMap, env, num::ParseIntError};
use thiserror::Error;

/// Runtime configuration for the API service.
#[derive(Clone)]
pub struct Config {
    pub database_url: SecretString,
    pub jwt_secret: SecretString,
    pub jwt_issuer: String,
    pub jwt_default_ttl_seconds: u64,
    pub jwt_max_ttl_seconds: u64,
    pub wallet_challenge_ttl_seconds: u64,
    pub trader_secrets: HashMap<String, SecretString>,
    pub server_port: u16,
}

impl Config {
    /// Load configuration from environment variables, honoring values supplied
    /// via a `.env` file when present.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url =
            env::var("DATABASE_URL").map_err(|_| ConfigError::Missing("DATABASE_URL"))?;
        let jwt_secret = env::var("JWT_SECRET").map_err(|_| ConfigError::Missing("JWT_SECRET"))?;

        let server_port = parse_server_port(env::var("SERVER_PORT").ok())?;
        let jwt_issuer = env::var("JWT_ISSUER").unwrap_or_else(|_| "dex-os-api".to_string());
        let jwt_default_ttl_seconds = parse_u64("JWT_TTL_SECONDS", 900)?;
        let jwt_max_ttl_seconds = parse_u64("JWT_MAX_TTL_SECONDS", 3600)?;
        let wallet_challenge_ttl_seconds = parse_u64("WALLET_CHALLENGE_TTL_SECONDS", 300)?;
        let trader_secrets = parse_trader_secrets(env::var("TRADER_SECRETS").ok())?;

        Ok(Self {
            database_url: SecretString::from(database_url),
            jwt_secret: SecretString::from(jwt_secret),
            jwt_issuer,
            jwt_default_ttl_seconds: jwt_default_ttl_seconds.max(60),
            jwt_max_ttl_seconds: jwt_max_ttl_seconds.max(jwt_default_ttl_seconds),
            wallet_challenge_ttl_seconds: wallet_challenge_ttl_seconds.max(60),
            trader_secrets,
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
    #[error("invalid numeric value for {var}: {err}")]
    InvalidNumber {
        var: &'static str,
        err: ParseIntError,
    },
    #[error("invalid TRADER_SECRETS entry '{entry}', expected trader:secret")]
    InvalidTraderSecret { entry: String },
}

fn parse_u64(var: &'static str, default: u64) -> Result<u64, ConfigError> {
    match env::var(var) {
        Ok(value) => value
            .parse::<u64>()
            .map_err(|err| ConfigError::InvalidNumber { var, err }),
        Err(_) => Ok(default),
    }
}

fn parse_trader_secrets(raw: Option<String>) -> Result<HashMap<String, SecretString>, ConfigError> {
    let mut map = HashMap::new();
    if let Some(raw) = raw {
        for entry in raw.split(',') {
            if entry.trim().is_empty() {
                continue;
            }
            let mut parts = entry.splitn(2, ':').map(|p| p.trim().to_string());
            let trader = parts.next().unwrap_or_default();
            let secret = parts
                .next()
                .ok_or_else(|| ConfigError::InvalidTraderSecret {
                    entry: entry.to_string(),
                })?;
            if trader.is_empty() || secret.is_empty() {
                return Err(ConfigError::InvalidTraderSecret {
                    entry: entry.to_string(),
                });
            }
            map.insert(trader, SecretString::from(secret));
        }
    }
    Ok(map)
}
