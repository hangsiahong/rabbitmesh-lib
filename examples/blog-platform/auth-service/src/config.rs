pub const SERVICE_NAME: &str = "auth-service";
pub const RABBITMQ_URL: &str = "amqp://localhost:5672";

// JWT secret (in production, use environment variable)
pub const JWT_SECRET: &str = "your-secret-key-change-in-production";
pub const TOKEN_EXPIRY_HOURS: i64 = 24;