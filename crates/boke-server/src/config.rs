use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_address: String,
    pub static_dir: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            bind_address: env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            static_dir: env::var("STATIC_DIR").unwrap_or_else(|_| "./static".to_string()),
        })
    }
}
