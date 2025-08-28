use serde::{Deserialize, Serialize};

/// Internal Post model for post service database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub title: String,
    pub content: String,
    pub author_id: String,
    pub author_username: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_published: bool,
    pub tags: Vec<String>,
}

impl Post {
    /// Convert to summary for listings
    pub fn to_summary(&self) -> PostSummary {
        PostSummary {
            id: self.id.clone(),
            title: self.title.clone(),
            author_username: self.author_username.clone(),
            created_at: self.created_at.clone(),
            tags: self.tags.clone(),
        }
    }

    /// Check if post exists and is published (for service-to-service calls)
    pub fn to_reference(&self) -> blog_common::PostReference {
        blog_common::PostReference {
            id: self.id.clone(),
            title: self.title.clone(),
            author_id: self.author_id.clone(),
            is_published: self.is_published,
        }
    }
}