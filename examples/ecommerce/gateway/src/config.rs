use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: Server,
    pub rabbitmq: RabbitMQ,
    pub gateway: Gateway,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub bind_address: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct RabbitMQ {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Gateway {
    pub name: String,
    pub version: String,
    pub enable_graphql: bool,
    pub enable_swagger: bool,
    pub cors_origins: Vec<String>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut builder = Config::builder()
            .set_default("server.bind_address", "0.0.0.0:3333")?
            .set_default("server.port", 3333)?
            .set_default("rabbitmq.url", "amqp://guest:guest@localhost:5672/%2f")?
            .set_default("gateway.name", "rabbitmesh-gateway")?
            .set_default("gateway.version", "0.1.0")?
            .set_default("gateway.enable_graphql", true)?
            .set_default("gateway.enable_swagger", true)?
            .set_default("gateway.cors_origins", vec!["*".to_string()])?;

        // Add configuration from file if exists
        if std::path::Path::new("config.toml").exists() {
            builder = builder.add_source(File::with_name("config"));
        }

        // Add environment variables with prefix
        builder = builder.add_source(Environment::with_prefix("GATEWAY"));

        let config = builder.build()?;
        config.try_deserialize()
    }
}