use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request to check if a post exists (for service-to-service calls)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PostExistsRequest {
    pub post_id: String,
}

/// Response for post existence check
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PostExistsResponse {
    pub exists: bool,
    pub post_id: String,
    pub title: Option<String>,      // Only if exists
    pub author_id: Option<String>,  // Only if exists
}

/// Minimal post info for cross-service references
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PostReference {
    pub id: String,
    pub title: String,
    pub author_id: String,
    pub is_published: bool,
}