use crate::models::*;
use crate::utils::*;
use rabbitmesh::{Message, MicroService, RpcResponse, ServiceClient};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

pub type CommentDatabase = Arc<RwLock<HashMap<String, Comment>>>;

pub async fn setup_sample_comments(db: &CommentDatabase) -> Result<(), Box<dyn std::error::Error>> {
    let mut comments = db.write().await;
    
    let comment1_id = generate_comment_id();
    comments.insert(comment1_id.clone(), Comment {
        id: comment1_id.clone(),
        post_id: "sample-post-1".to_string(),
        author_id: "sample-user-1".to_string(),
        author_username: "john_doe".to_string(),
        content: "Great post! Looking forward to more content.".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        is_approved: true,
    });

    let comment2_id = generate_comment_id();
    comments.insert(comment2_id.clone(), Comment {
        id: comment2_id.clone(),
        post_id: "sample-post-1".to_string(),
        author_id: "sample-user-2".to_string(),
        author_username: "jane_smith".to_string(),
        content: "Thanks for sharing this! Very helpful.".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        is_approved: true,
    });

    info!("✅ Sample comments created");
    Ok(())
}

pub async fn register_comment_handlers(
    service: &MicroService,
    client: Arc<ServiceClient>,
    db: CommentDatabase,
) -> Result<(), Box<dyn std::error::Error>> {
    
    // Create comment handler - requires authentication and post verification
    let db_clone = db.clone();
    let client_clone = client.clone();
    service.register_function("create_comment", move |msg: Message| {
        let db = db_clone.clone();
        let client = client_clone.clone();
        async move {
            let request: CreateCommentRequest = msg.deserialize_payload()?;
            info!("💬 Creating comment for post: {}", request.post_id);
            
            // First, validate user token with auth service
            let user = match validate_user_token(&client, &request.token).await {
                Ok(user) => user,
                Err(e) => {
                    error!("❌ Authentication failed: {}", e);
                    return Ok(RpcResponse::error(format!("Authentication failed: {}", e)));
                }
            };
            
            // Then, verify the post exists with post service
            let _post = match verify_post_exists(&client, &request.post_id).await {
                Ok(post) => post,
                Err(e) => {
                    error!("❌ Post verification failed: {}", e);
                    return Ok(RpcResponse::error(format!("Post verification failed: {}", e)));
                }
            };
            
            // Create the comment
            let comment_id = generate_comment_id();
            let now = chrono::Utc::now().to_rfc3339();
            
            let comment = Comment {
                id: comment_id.clone(),
                post_id: request.post_id,
                author_id: user.id,
                author_username: user.username,
                content: request.content,
                created_at: now.clone(),
                updated_at: now,
                is_approved: true, // Auto-approve for demo
            };
            
            let mut comments = db.write().await;
            comments.insert(comment_id, comment.clone());
            
            info!("✅ Comment created successfully by: {}", comment.author_username);
            Ok(RpcResponse::success(comment, 25)?) // Higher latency due to multi-service calls
        }
    }).await;

    // Update comment handler - requires authentication and ownership
    let db_clone = db.clone();
    let client_clone = client.clone();
    service.register_function("update_comment", move |msg: Message| {
        let db = db_clone.clone();
        let client = client_clone.clone();
        async move {
            let request: UpdateCommentRequest = msg.deserialize_payload()?;
            info!("✏️ Updating comment: {}", request.comment_id);
            
            // Validate user token with auth service
            let user = match validate_user_token(&client, &request.token).await {
                Ok(user) => user,
                Err(e) => {
                    error!("❌ Authentication failed: {}", e);
                    return Ok(RpcResponse::error(format!("Authentication failed: {}", e)));
                }
            };
            
            let mut comments = db.write().await;
            
            match comments.get_mut(&request.comment_id) {
                Some(comment) => {
                    // Check if user owns the comment
                    if comment.author_id != user.id {
                        return Ok(RpcResponse::error("You can only update your own comments".to_string()));
                    }
                    
                    // Update the comment
                    comment.content = request.content;
                    comment.updated_at = chrono::Utc::now().to_rfc3339();
                    
                    info!("✅ Comment updated successfully");
                    Ok(RpcResponse::success(comment.clone(), 15)?)
                }
                None => {
                    Ok(RpcResponse::error("Comment not found".to_string()))
                }
            }
        }
    }).await;

    // Get comments for a specific post
    let db_clone = db.clone();
    service.register_function("get_comments_by_post", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            let request: GetCommentsByPostRequest = msg.deserialize_payload()?;
            info!("📋 Getting comments for post: {}", request.post_id);
            
            let comments = db.read().await;
            let post_comments: Vec<Comment> = comments.values()
                .filter(|c| c.post_id == request.post_id && c.is_approved)
                .cloned()
                .collect();
            
            info!("✅ Found {} approved comments for post", post_comments.len());
            Ok(RpcResponse::success(post_comments, 10)?)
        }
    }).await;

    // Get all comments (for admin purposes)
    let db_clone = db.clone();
    service.register_function("list_comments", move |_msg: Message| {
        let db = db_clone.clone();
        async move {
            info!("📋 Listing all comments");
            
            let comments = db.read().await;
            let comment_list: Vec<Comment> = comments.values()
                .filter(|c| c.is_approved)
                .cloned()
                .collect();
            
            info!("✅ Listed {} approved comments", comment_list.len());
            Ok(RpcResponse::success(comment_list, 10)?)
        }
    }).await;

    // Get comment by ID
    let db_clone = db.clone();
    service.register_function("get_comment", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            let comment_id: String = msg.deserialize_payload()?;
            info!("🔍 Getting comment: {}", comment_id);
            
            let comments = db.read().await;
            match comments.get(&comment_id) {
                Some(comment) if comment.is_approved => {
                    info!("✅ Found comment by: {}", comment.author_username);
                    Ok(RpcResponse::success(comment.clone(), 5)?)
                }
                Some(_) => {
                    Ok(RpcResponse::error("Comment not approved".to_string()))
                }
                None => {
                    Ok(RpcResponse::error("Comment not found".to_string()))
                }
            }
        }
    }).await;

    // Health check
    service.register_function("ping", |_msg: Message| async move {
        Ok(RpcResponse::success("Comment service is healthy", 1)?)
    }).await;

    Ok(())
}