module.exports = {
  apps: [
    {
      name: 'rabbitmesh-user-service',
      script: './target/release/user-service',
      cwd: './user-service',
      instances: 2,
      exec_mode: 'fork',
      autorestart: true,
      watch: false,
      max_memory_restart: '1G',
      env: {
        RUST_LOG: 'user_service=info,rabbitmesh=info',
        USER_SERVICE_DATABASE_URL: 'mongodb://localhost:27017',
        USER_SERVICE_DATABASE_NAME: 'ecommerce',
        USER_SERVICE_RABBITMQ_URL: 'amqp://guest:guest@localhost:5672/%2f',
        USER_SERVICE_SERVICE_NAME: 'user-service',
        USER_SERVICE_SERVICE_VERSION: '0.1.0'
      },
      env_production: {
        RUST_LOG: 'user_service=warn,rabbitmesh=warn',
        USER_SERVICE_DATABASE_URL: 'mongodb://mongodb:27017',
        USER_SERVICE_RABBITMQ_URL: 'amqp://guest:guest@rabbitmq:5672/%2f'
      },
      log_file: './logs/user-service.log',
      error_file: './logs/user-service-error.log',
      out_file: './logs/user-service-out.log',
      log_date_format: 'YYYY-MM-DD HH:mm:ss Z'
    },
    
    {
      name: 'rabbitmesh-auth-service',
      script: './target/release/auth-service',
      cwd: './auth-service',
      instances: 2,
      exec_mode: 'fork',
      autorestart: true,
      watch: false,
      max_memory_restart: '512M',
      env: {
        RUST_LOG: 'auth_service=info,rabbitmesh=info',
        AUTH_SERVICE_JWT_SECRET: 'your-super-secret-jwt-key-change-in-production',
        AUTH_SERVICE_JWT_EXPIRATION_HOURS: '24',
        AUTH_SERVICE_JWT_REFRESH_EXPIRATION_DAYS: '30',
        AUTH_SERVICE_RABBITMQ_URL: 'amqp://guest:guest@localhost:5672/%2f',
        AUTH_SERVICE_SERVICE_NAME: 'auth-service',
        AUTH_SERVICE_SERVICE_VERSION: '0.1.0',
        AUTH_SERVICE_USER_SERVICE_URL: 'user-service'
      },
      env_production: {
        RUST_LOG: 'auth_service=warn,rabbitmesh=warn',
        AUTH_SERVICE_JWT_SECRET: process.env.JWT_SECRET || 'change-this-in-production',
        AUTH_SERVICE_RABBITMQ_URL: 'amqp://guest:guest@rabbitmq:5672/%2f'
      },
      log_file: './logs/auth-service.log',
      error_file: './logs/auth-service-error.log',
      out_file: './logs/auth-service-out.log',
      log_date_format: 'YYYY-MM-DD HH:mm:ss Z'
    },

    {
      name: 'rabbitmesh-order-service',
      script: './target/release/order-service',
      cwd: './order-service',
      instances: 3,
      exec_mode: 'fork',
      autorestart: true,
      watch: false,
      max_memory_restart: '1G',
      env: {
        RUST_LOG: 'order_service=info,rabbitmesh=info',
        ORDER_SERVICE_DATABASE_URL: 'mongodb://localhost:27017',
        ORDER_SERVICE_DATABASE_NAME: 'ecommerce',
        ORDER_SERVICE_REDIS_URL: 'redis://localhost:6379',
        ORDER_SERVICE_REDIS_CACHE_TTL_SECONDS: '300',
        ORDER_SERVICE_RABBITMQ_URL: 'amqp://guest:guest@localhost:5672/%2f',
        ORDER_SERVICE_SERVICE_NAME: 'order-service',
        ORDER_SERVICE_SERVICE_VERSION: '0.1.0'
      },
      env_production: {
        RUST_LOG: 'order_service=warn,rabbitmesh=warn',
        ORDER_SERVICE_DATABASE_URL: 'mongodb://mongodb:27017',
        ORDER_SERVICE_REDIS_URL: 'redis://redis:6379',
        ORDER_SERVICE_RABBITMQ_URL: 'amqp://guest:guest@rabbitmq:5672/%2f'
      },
      log_file: './logs/order-service.log',
      error_file: './logs/order-service-error.log',
      out_file: './logs/order-service-out.log',
      log_date_format: 'YYYY-MM-DD HH:mm:ss Z'
    },

    {
      name: 'rabbitmesh-gateway',
      script: './target/release/gateway',
      cwd: './gateway',
      instances: 1,
      exec_mode: 'fork',
      autorestart: true,
      watch: false,
      max_memory_restart: '512M',
      env: {
        RUST_LOG: 'gateway=info,rabbitmesh_gateway=info,tower_http=info',
        GATEWAY_SERVER_BIND_ADDRESS: '0.0.0.0:3000',
        GATEWAY_SERVER_PORT: '3000',
        GATEWAY_RABBITMQ_URL: 'amqp://guest:guest@localhost:5672/%2f',
        GATEWAY_GATEWAY_NAME: 'rabbitmesh-gateway',
        GATEWAY_GATEWAY_VERSION: '0.1.0',
        GATEWAY_GATEWAY_ENABLE_GRAPHQL: 'true',
        GATEWAY_GATEWAY_ENABLE_SWAGGER: 'true'
      },
      env_production: {
        RUST_LOG: 'gateway=warn,rabbitmesh_gateway=warn',
        GATEWAY_RABBITMQ_URL: 'amqp://guest:guest@rabbitmq:5672/%2f'
      },
      log_file: './logs/gateway.log',
      error_file: './logs/gateway-error.log',
      out_file: './logs/gateway-out.log',
      log_date_format: 'YYYY-MM-DD HH:mm:ss Z'
    }
  ],

  // Deployment configurations
  deploy: {
    production: {
      user: 'deploy',
      host: ['production-server.com'],
      ref: 'origin/main',
      repo: 'git@github.com:your-org/rabbitmesh-ecommerce.git',
      path: '/var/www/rabbitmesh-ecommerce',
      'post-deploy': 'cargo build --release && pm2 reload ecosystem.config.js --env production && pm2 save'
    },
    
    staging: {
      user: 'deploy',
      host: ['staging-server.com'],
      ref: 'origin/develop',
      repo: 'git@github.com:your-org/rabbitmesh-ecommerce.git',
      path: '/var/www/rabbitmesh-ecommerce-staging',
      'post-deploy': 'cargo build --release && pm2 reload ecosystem.config.js --env staging && pm2 save'
    }
  },

  // PM2 Plus monitoring
  pmx: {
    monitor: {
      network: true,
      ports: true
    }
  }
};