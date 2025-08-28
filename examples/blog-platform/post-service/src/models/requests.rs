use serde::Deserialize;
use utoipa::ToSchema;

/// Request to create a new post
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
    pub token: String,  // For auth verification
    pub tags: Vec<String>,
}

/// Request to update an existing post
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePostRequest {
    pub post_id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub token: String,  // For auth verification
    pub tags: Option<Vec<String>>,
}

/// Request to get posts by author
#[derive(Debug, Deserialize, ToSchema)]
pub struct GetPostsByAuthorRequest {
    pub author_id: String,
}