use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use utoipa::ToSchema;

/// Todo item
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new todo
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: Option<String>,
}

/// Request to update a todo
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateTodoRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
}

/// Response for todo operations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TodoResponse {
    pub success: bool,
    pub message: String,
    pub todo: Option<Todo>,
}

/// Response for list operations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TodoListResponse {
    pub success: bool,
    pub message: String,
    pub todos: Vec<Todo>,
    pub total: usize,
}

impl Todo {
    pub fn new(title: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            completed: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update(&mut self, request: UpdateTodoRequest) {
        if let Some(title) = request.title {
            self.title = title;
        }
        if let Some(description) = request.description {
            self.description = Some(description);
        }
        if let Some(completed) = request.completed {
            self.completed = completed;
        }
        self.updated_at = Utc::now();
    }
}