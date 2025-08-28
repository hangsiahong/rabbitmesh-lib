# Simple Todo List Microservice

A complete example demonstrating the RabbitMesh framework with macro-based service definitions and auto-generated API gateway.

## 🎯 What This Example Demonstrates

This is the **correct** way to use RabbitMesh:

1. **Macro-Based Service Definition**: Services defined with `#[service_definition]` and `#[service_impl]`
2. **Auto-Generated Routes**: HTTP routes automatically extracted from `#[service_method]` annotations
3. **Zero-Port Architecture**: Services communicate only via RabbitMQ, no HTTP ports between services
4. **Auto-Generated Gateway**: API Gateway automatically discovers and proxies to services

## 🏗️ Architecture

```
Frontend/Client  →  HTTP  →  Auto-Generated Gateway  →  RabbitMQ  →  Todo Service
                              (Auto-Discovery)                      (Macro-Based)
```

## 📋 Service Definition (The RIGHT Way)

```rust
#[service_definition]
pub struct TodoService;

#[service_impl]
impl TodoService {
    #[service_method("POST /todos")]
    pub async fn create_todo(request: CreateTodoRequest) -> Result<TodoResponse, String> {
        // Just your business logic!
        // Framework handles everything else
    }

    #[service_method("GET /todos/:id")]
    pub async fn get_todo(todo_id: String) -> Result<TodoResponse, String> {
        // Clean, simple method signatures
    }
}
```

## 🚀 Running the Example

### 1. Start RabbitMQ

```bash
docker run -d --name rabbitmq -p 5672:5672 -p 15672:15672 rabbitmq:3.12-management
```

### 2. Start Todo Service

```bash
cd examples/simple-todo
RABBITMQ_URL=amqp://guest:guest@localhost:5672/%2f cargo run --bin todo-service
```

### 3. Start Auto-Generated Gateway

```bash
# In another terminal
HTTP_PORT=3000 RABBITMQ_URL=amqp://guest:guest@localhost:5672/%2f cargo run --bin todo-gateway
```

## 🧪 Testing the API

### Create a Todo
```bash
curl -X POST http://localhost:3000/todos \
  -H "Content-Type: application/json" \
  -d '{"title": "Learn RabbitMesh", "description": "Master zero-port microservices"}'
```

### List Todos
```bash
curl http://localhost:3000/todos
```

### Get a Todo
```bash
curl http://localhost:3000/todos/{todo-id}
```

### Update a Todo
```bash
curl -X PUT http://localhost:3000/todos/{todo-id} \
  -H "Content-Type: application/json" \
  -d '{"completed": true}'
```

### Delete a Todo
```bash
curl -X DELETE http://localhost:3000/todos/{todo-id}
```

### Complete a Todo
```bash
curl -X POST http://localhost:3000/todos/{todo-id}/complete
```

### Get Statistics
```bash
curl http://localhost:3000/todos/stats
```

## ✨ Key Features Demonstrated

### 1. Macro-Based Definition
- `#[service_definition]` - Marks the service struct
- `#[service_impl]` - Processes the impl block
- `#[service_method("HTTP_METHOD /path")]` - Defines endpoints

### 2. Automatic Features
- **Route Discovery**: HTTP routes auto-extracted from method annotations
- **RPC Registration**: Methods automatically registered as RPC handlers  
- **Serialization**: JSON serialization/deserialization handled automatically
- **Error Handling**: Structured error responses
- **Service Registry**: Automatic service registration with RabbitMQ

### 3. Clean Method Signatures
```rust
// Simple parameter
pub async fn get_todo(todo_id: String) -> Result<TodoResponse, String>

// Complex parameter
pub async fn create_todo(request: CreateTodoRequest) -> Result<TodoResponse, String>

// Path parameter + body (tuple)
pub async fn update_todo(params: (String, UpdateTodoRequest)) -> Result<TodoResponse, String>
```

### 4. Auto-Generated Gateway
The gateway automatically:
- Discovers services via RabbitMQ
- Maps HTTP routes to RPC calls
- Handles request/response serialization
- Provides service health checks

## 🔍 What Makes This Different

### ❌ Wrong Approach (Complex Manual Setup)
```rust
// Manually registering functions (old way)
service.register_function("create_todo", |msg| async move {
    // Manual message handling, serialization, etc.
});
```

### ✅ Right Approach (Macro Magic)
```rust
// Clean, declarative service definition
#[service_method("POST /todos")]
pub async fn create_todo(request: CreateTodoRequest) -> Result<TodoResponse, String> {
    // Just business logic!
}
```

## 📊 Service Output

When you start the service, you'll see:
```
✅ Todo Service started successfully
🎯 Service name: TodoService
📊 Auto-discovered 6 HTTP routes:
   POST -> /todos
   GET -> /todos/:id
   GET -> /todos
   PUT -> /todos/:id
   DELETE -> /todos/:id
   POST -> /todos/:id/complete
   GET -> /todos/stats
```

## 🎯 Benefits

1. **Zero Configuration**: Routes auto-discovered from code
2. **Type Safety**: Full Rust type checking
3. **Clean Code**: Just business logic, no boilerplate
4. **Automatic Documentation**: Routes self-documenting
5. **Service Discovery**: Gateway finds services automatically
6. **Zero Ports**: No HTTP servers between services

This is the **correct** RabbitMesh usage pattern!