use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: Database,
    pub redis: Redis,
    pub rabbitmq: RabbitMQ,
    pub service: Service,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub url: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Redis {
    pub url: String,
    pub cache_ttl_seconds: u64,
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

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut builder = Config::builder()
            .set_default("database.url", "mongodb://localhost:27017")?
            .set_default("database.name", "ecommerce")?
            .set_default("redis.url", "redis://localhost:6379")?
            .set_default("redis.cache_ttl_seconds", 300)?
            .set_default("rabbitmq.url", "amqp://guest:guest@localhost:5672/%2f")?
            .set_default("service.name", "order-service")?
            .set_default("service.version", "0.1.0")?;

        // Add configuration from file if exists
        if std::path::Path::new("config.toml").exists() {
            builder = builder.add_source(File::with_name("config"));
        }

        // Add environment variables with prefix
        builder = builder.add_source(Environment::with_prefix("ORDER_SERVICE"));

        let config = builder.build()?;
        config.try_deserialize()
    }
}