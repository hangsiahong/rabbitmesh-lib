use serde::Deserialize;
use utoipa::ToSchema;

/// Request to create a new comment
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCommentRequest {
    pub post_id: String,
    pub content: String,
    pub token: String,  // For auth verification
}

/// Request to update a comment
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCommentRequest {
    pub comment_id: String,
    pub content: String,
    pub token: String,  // For auth verification
}

/// Request to get comments by post
#[derive(Debug, Deserialize, ToSchema)]
pub struct GetCommentsByPostRequest {
    pub post_id: String,
}