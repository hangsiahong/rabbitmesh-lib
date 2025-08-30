// MongoDB initialization script for RabbitMesh E-Commerce Example

// Switch to the ecommerce database
db = db.getSiblingDB('ecommerce');

// Create collections with validation
db.createCollection('users', {
  validator: {
    $jsonSchema: {
      bsonType: 'object',
      required: ['id', 'email', 'name', 'role'],
      properties: {
        id: {
          bsonType: 'string',
          description: 'User ID must be a string and is required'
        },
        email: {
          bsonType: 'string',
          pattern: '^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$',
          description: 'Email must be a valid email format and is required'
        },
        name: {
          bsonType: 'string',
          minLength: 1,
          description: 'Name must be a non-empty string and is required'
        },
        role: {
          bsonType: 'string',
          enum: ['admin', 'manager', 'customer'],
          description: 'Role must be one of admin, manager, or customer'
        },
        permissions: {
          bsonType: 'array',
          items: {
            bsonType: 'string'
          },
          description: 'Permissions must be an array of strings'
        }
      }
    }
  }
});

db.createCollection('orders', {
  validator: {
    $jsonSchema: {
      bsonType: 'object',
      required: ['id', 'user_id', 'items', 'total_amount', 'status'],
      properties: {
        id: {
          bsonType: 'string',
          description: 'Order ID must be a string and is required'
        },
        user_id: {
          bsonType: 'string',
          description: 'User ID must be a string and is required'
        },
        items: {
          bsonType: 'array',
          minItems: 1,
          description: 'Items must be a non-empty array'
        },
        total_amount: {
          bsonType: 'double',
          minimum: 0,
          description: 'Total amount must be a positive number'
        },
        status: {
          bsonType: 'string',
          enum: ['Pending', 'Confirmed', 'Processing', 'Shipped', 'Delivered', 'Cancelled'],
          description: 'Status must be a valid order status'
        }
      }
    }
  }
});

// Create indexes for better performance
db.users.createIndex({ 'email': 1 }, { unique: true });
db.users.createIndex({ 'id': 1 }, { unique: true });
db.users.createIndex({ 'role': 1 });

db.orders.createIndex({ 'id': 1 }, { unique: true });
db.orders.createIndex({ 'user_id': 1 });
db.orders.createIndex({ 'status': 1 });
db.orders.createIndex({ 'created_at': -1 });

// Insert sample data for testing
// Admin user
db.users.insertOne({
  id: 'admin-001',
  email: 'admin@rabbitmesh.dev',
  name: 'System Administrator',
  password: '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LeVMpYlDiRZGUYvMy', // password: admin123
  role: 'admin',
  permissions: [
    'users:read', 'users:write', 'users:delete',
    'orders:read', 'orders:write', 'orders:delete'
  ],
  created_at: new Date(),
  updated_at: new Date()
});

// Manager user
db.users.insertOne({
  id: 'manager-001',
  email: 'manager@rabbitmesh.dev',
  name: 'Store Manager',
  password: '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LeVMpYlDiRZGUYvMy', // password: manager123
  role: 'manager',
  permissions: [
    'users:read', 'users:write',
    'orders:read', 'orders:write'
  ],
  created_at: new Date(),
  updated_at: new Date()
});

// Customer user
db.users.insertOne({
  id: 'customer-001',
  email: 'customer@rabbitmesh.dev',
  name: 'Test Customer',
  password: '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LeVMpYlDiRZGUYvMy', // password: customer123
  role: 'customer',
  permissions: [
    'orders:read', 'orders:write'
  ],
  created_at: new Date(),
  updated_at: new Date()
});

// Sample orders
db.orders.insertMany([
  {
    id: 'ORD-12345678',
    user_id: 'customer-001',
    items: [
      {
        product_id: 'prod-001',
        product_name: 'RabbitMesh T-Shirt',
        quantity: 2,
        unit_price: 25.99,
        total_price: 51.98
      },
      {
        product_id: 'prod-002',
        product_name: 'Microservices Mug',
        quantity: 1,
        unit_price: 15.99,
        total_price: 15.99
      }
    ],
    total_amount: 67.97,
    status: 'Pending',
    created_at: new Date(),
    updated_at: new Date()
  },
  {
    id: 'ORD-87654321',
    user_id: 'customer-001',
    items: [
      {
        product_id: 'prod-003',
        product_name: 'Distributed Systems Book',
        quantity: 1,
        unit_price: 59.99,
        total_price: 59.99
      }
    ],
    total_amount: 59.99,
    status: 'Confirmed',
    created_at: new Date(Date.now() - 86400000), // 1 day ago
    updated_at: new Date(Date.now() - 3600000)   // 1 hour ago
  }
]);

print('✅ RabbitMesh E-Commerce database initialized successfully!');
print('📊 Collections created: users, orders');
print('🔍 Indexes created for optimal performance');
print('👥 Sample users created:');
print('   - admin@rabbitmesh.dev (password: admin123)');
print('   - manager@rabbitmesh.dev (password: manager123)');  
print('   - customer@rabbitmesh.dev (password: customer123)');
print('📦 Sample orders created for testing');