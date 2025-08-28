use serde::Serialize;
use utoipa::ToSchema;

/// Post summary for list views
#[derive(Debug, Serialize, ToSchema)]
pub struct PostSummary {
    pub id: String,
    pub title: String,
    pub author_username: String,
    pub created_at: String,
    pub tags: Vec<String>,
}