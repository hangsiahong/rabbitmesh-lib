use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub jwt: JwtSettings,
    pub rabbitmq: RabbitMQ,
    pub service: Service,
    pub user_service: UserService,
}

#[derive(Debug, Deserialize)]
pub struct JwtSettings {
    pub secret: String,
    pub expiration_hours: u64,
    pub refresh_expiration_days: u64,
}

#[derive(Debug, Deserialize)]
pub struct RabbitMQ {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Service {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct UserService {
    pub url: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut builder = Config::builder()
            .set_default("jwt.secret", "your-super-secret-jwt-key-change-in-production")?
            .set_default("jwt.expiration_hours", 24)?
            .set_default("jwt.refresh_expiration_days", 30)?
            .set_default("rabbitmq.url", "amqp://guest:guest@localhost:5672/%2f")?
            .set_default("service.name", "auth-service")?
            .set_default("service.version", "0.1.0")?
            .set_default("user_service.url", "user-service")?;

        // Add configuration from file if exists
        if std::path::Path::new("config.toml").exists() {
            builder = builder.add_source(File::with_name("config"));
        }

        // Add environment variables with prefix
        builder = builder.add_source(Environment::with_prefix("AUTH_SERVICE"));

        let config = builder.build()?;
        config.try_deserialize()
    }
}