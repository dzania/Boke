use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_address: String,
    pub static_dir: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = Self::build_database_url()?;

        Ok(Self {
            database_url,
            bind_address: env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            static_dir: env::var("STATIC_DIR").unwrap_or_else(|_| "./static".to_string()),
        })
    }

    fn build_database_url() -> Result<String, ConfigError> {
        // Determine database type: "postgres" or "sqlite" (default: sqlite)
        let db_type = env::var("DB_TYPE").unwrap_or_else(|_| "sqlite".to_string());

        match db_type.to_lowercase().as_str() {
            "postgres" | "postgresql" => {
                let host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
                let port = env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
                let name = env::var("DB_NAME").unwrap_or_else(|_| "boke".to_string());
                let user = env::var("DB_USER").unwrap_or_else(|_| "boke".to_string());
                let password = env::var("DB_PASSWORD").ok();

                let url = match password {
                    Some(pass) => {
                        format!("postgres://{}:{}@{}:{}/{}", user, pass, host, port, name)
                    }
                    None => format!("postgres://{}@{}:{}/{}", user, host, port, name),
                };

                Ok(url)
            }
            "sqlite" => {
                let path = env::var("DB_PATH").unwrap_or_else(|_| "/data/boke.db".to_string());
                Ok(format!("sqlite://{}", path))
            }
            other => Err(ConfigError::InvalidDbType(other.to_string())),
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidDbType(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidDbType(t) => {
                write!(f, "Invalid DB_TYPE '{}'. Use 'sqlite' or 'postgres'", t)
            }
        }
    }
}

impl std::error::Error for ConfigError {}
