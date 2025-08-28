use serde::{Deserialize, Serialize};

/// Internal Comment model for comment service database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub post_id: String,
    pub author_id: String,
    pub author_username: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_approved: bool,
}

impl Comment {
    /// Convert to external reference for cross-service communication
    pub fn to_reference(&self) -> blog_common::CommentReference {
        blog_common::CommentReference {
            id: self.id.clone(),
            post_id: self.post_id.clone(),
            author_id: self.author_id.clone(),
            content: self.content.clone(),
            is_approved: self.is_approved,
            created_at: self.created_at.clone(),
        }
    }
}