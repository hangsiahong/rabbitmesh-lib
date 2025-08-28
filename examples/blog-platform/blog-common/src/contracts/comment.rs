use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Minimal comment info for cross-service references
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CommentReference {
    pub id: String,
    pub post_id: String,
    pub author_id: String,
    pub content: String,
    pub is_approved: bool,
    pub created_at: String,
}