use rabbitmesh_macros::{service_definition, service_impl};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use anyhow::Result;

use crate::models::*;

// In-memory storage for simplicity (in production, use a database)
type TodoStorage = Arc<RwLock<HashMap<String, Todo>>>;

#[service_definition]
pub struct TodoService;

static TODO_STORAGE: tokio::sync::OnceCell<TodoStorage> = tokio::sync::OnceCell::const_new();

impl TodoService {
    pub async fn init() -> Result<()> {
        let storage = Arc::new(RwLock::new(HashMap::new()));
        TODO_STORAGE.set(storage)
            .map_err(|_| anyhow::anyhow!("Storage already initialized"))?;
        Ok(())
    }

    fn get_storage() -> &'static TodoStorage {
        TODO_STORAGE.get().expect("Storage not initialized")
    }
}

#[service_impl]
impl TodoService {
    /// Create a new todo item
    #[service_method("POST /todos")]
    pub async fn create_todo(request: CreateTodoRequest) -> Result<TodoResponse, String> {
        info!("📝 Creating new todo: {}", request.title);
        
        let todo = Todo::new(request.title, request.description);
        let todo_id = todo.id.clone();
        
        // Store the todo
        let storage = Self::get_storage();
        let mut storage = storage.write().await;
        storage.insert(todo_id.clone(), todo.clone());
        
        info!("✅ Todo created successfully: {}", todo_id);
        
        Ok(TodoResponse {
            success: true,
            message: "Todo created successfully".to_string(),
            todo: Some(todo),
        })
    }

    /// Get a todo by ID
    #[service_method("GET /todos/:id")]
    pub async fn get_todo(todo_id: String) -> Result<TodoResponse, String> {
        info!("🔍 Getting todo: {}", todo_id);
        
        let storage = Self::get_storage();
        let storage = storage.read().await;
        
        match storage.get(&todo_id) {
            Some(todo) => {
                info!("✅ Todo found: {}", todo_id);
                Ok(TodoResponse {
                    success: true,
                    message: "Todo retrieved successfully".to_string(),
                    todo: Some(todo.clone()),
                })
            }
            None => {
                warn!("❌ Todo not found: {}", todo_id);
                Err("Todo not found".to_string())
            }
        }
    }

    /// Get all todos
    #[service_method("GET /todos")]
    pub async fn list_todos() -> Result<TodoListResponse, String> {
        info!("📋 Listing all todos");
        
        let storage = Self::get_storage();
        let storage = storage.read().await;
        let todos: Vec<Todo> = storage.values().cloned().collect();
        let total = todos.len();
        
        info!("✅ Retrieved {} todos", total);
        
        Ok(TodoListResponse {
            success: true,
            message: format!("Retrieved {} todos", total),
            todos,
            total,
        })
    }

    /// Update a todo
    #[service_method("PUT /todos/:id")]
    pub async fn update_todo(params: (String, UpdateTodoRequest)) -> Result<TodoResponse, String> {
        let (todo_id, request) = params;
        info!("✏️ Updating todo: {}", todo_id);
        
        let storage = Self::get_storage();
        let mut storage = storage.write().await;
        
        match storage.get_mut(&todo_id) {
            Some(todo) => {
                todo.update(request);
                info!("✅ Todo updated successfully: {}", todo_id);
                
                Ok(TodoResponse {
                    success: true,
                    message: "Todo updated successfully".to_string(),
                    todo: Some(todo.clone()),
                })
            }
            None => {
                warn!("❌ Todo not found for update: {}", todo_id);
                Err("Todo not found".to_string())
            }
        }
    }

    /// Delete a todo
    #[service_method("DELETE /todos/:id")]
    pub async fn delete_todo(todo_id: String) -> Result<TodoResponse, String> {
        info!("🗑️ Deleting todo: {}", todo_id);
        
        let storage = Self::get_storage();
        let mut storage = storage.write().await;
        
        match storage.remove(&todo_id) {
            Some(todo) => {
                info!("✅ Todo deleted successfully: {}", todo_id);
                Ok(TodoResponse {
                    success: true,
                    message: "Todo deleted successfully".to_string(),
                    todo: Some(todo),
                })
            }
            None => {
                warn!("❌ Todo not found for deletion: {}", todo_id);
                Err("Todo not found".to_string())
            }
        }
    }

    /// Mark todo as completed
    #[service_method("POST /todos/:id/complete")]
    pub async fn complete_todo(todo_id: String) -> Result<TodoResponse, String> {
        info!("✅ Marking todo as completed: {}", todo_id);
        
        let storage = Self::get_storage();
        let mut storage = storage.write().await;
        
        match storage.get_mut(&todo_id) {
            Some(todo) => {
                todo.completed = true;
                todo.updated_at = chrono::Utc::now();
                info!("✅ Todo marked as completed: {}", todo_id);
                
                Ok(TodoResponse {
                    success: true,
                    message: "Todo marked as completed".to_string(),
                    todo: Some(todo.clone()),
                })
            }
            None => {
                warn!("❌ Todo not found for completion: {}", todo_id);
                Err("Todo not found".to_string())
            }
        }
    }

    /// Get statistics about todos
    #[service_method("GET /todos/stats")]
    pub async fn get_stats() -> Result<serde_json::Value, String> {
        info!("📊 Getting todo statistics");
        
        let storage = Self::get_storage();
        let storage = storage.read().await;
        let todos: Vec<&Todo> = storage.values().collect();
        
        let total = todos.len();
        let completed = todos.iter().filter(|todo| todo.completed).count();
        let pending = total - completed;
        
        let stats = serde_json::json!({
            "success": true,
            "message": "Statistics retrieved successfully",
            "stats": {
                "total": total,
                "completed": completed,
                "pending": pending,
                "completion_rate": if total > 0 { completed as f64 / total as f64 } else { 0.0 }
            }
        });
        
        info!("✅ Stats: {} total, {} completed, {} pending", total, completed, pending);
        
        Ok(stats)
    }
}