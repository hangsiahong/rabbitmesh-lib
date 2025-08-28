use crate::models::*;
use crate::utils::*;
use rabbitmesh::{Message, MicroService, RpcResponse, ServiceClient};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

pub type PostDatabase = Arc<RwLock<HashMap<String, Post>>>;

pub async fn setup_sample_posts(db: &PostDatabase) -> Result<(), Box<dyn std::error::Error>> {
    let mut posts = db.write().await;
    
    let post1_id = generate_post_id();
    posts.insert(post1_id.clone(), Post {
        id: post1_id.clone(),
        title: "Welcome to Our Blog!".to_string(),
        content: "This is our first blog post. Welcome to the platform!".to_string(),
        author_id: "sample-user-1".to_string(),
        author_username: "admin".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        is_published: true,
        tags: vec!["welcome".to_string(), "announcement".to_string()],
    });

    let post2_id = generate_post_id();
    posts.insert(post2_id.clone(), Post {
        id: post2_id.clone(),
        title: "Getting Started with RabbitMesh".to_string(),
        content: "RabbitMesh makes microservices communication simple and efficient.".to_string(),
        author_id: "sample-user-2".to_string(),
        author_username: "tech_writer".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        is_published: true,
        tags: vec!["tutorial".to_string(), "rabbitmesh".to_string()],
    });

    info!("✅ Sample posts created");
    Ok(())
}

pub async fn register_post_handlers(
    service: &MicroService,
    client: Arc<ServiceClient>,
    db: PostDatabase,
) -> Result<(), Box<dyn std::error::Error>> {
    
    // Create post handler - requires authentication
    let db_clone = db.clone();
    let client_clone = client.clone();
    service.register_function("create_post", move |msg: Message| {
        let db = db_clone.clone();
        let client = client_clone.clone();
        async move {
            let request: CreatePostRequest = msg.deserialize_payload()?;
            info!("📝 Creating post: {}", request.title);
            
            // Validate user token with auth service
            match validate_user_token(&client, &request.token).await {
                Ok(user) => {
                    let post_id = generate_post_id();
                    let now = chrono::Utc::now().to_rfc3339();
                    
                    let post = Post {
                        id: post_id.clone(),
                        title: request.title,
                        content: request.content,
                        author_id: user.id,
                        author_username: user.username,
                        created_at: now.clone(),
                        updated_at: now,
                        is_published: true,
                        tags: request.tags,
                    };
                    
                    let mut posts = db.write().await;
                    posts.insert(post_id, post.clone());
                    
                    info!("✅ Post created successfully: {}", post.title);
                    Ok(RpcResponse::success(post, 20)?)
                }
                Err(e) => {
                    error!("❌ Authentication failed: {}", e);
                    Ok(RpcResponse::error(format!("Authentication failed: {}", e)))
                }
            }
        }
    }).await;

    // Update post handler - requires authentication and ownership
    let db_clone = db.clone();
    let client_clone = client.clone();
    service.register_function("update_post", move |msg: Message| {
        let db = db_clone.clone();
        let client = client_clone.clone();
        async move {
            let request: UpdatePostRequest = msg.deserialize_payload()?;
            info!("✏️ Updating post: {}", request.post_id);
            
            // Validate user token with auth service
            match validate_user_token(&client, &request.token).await {
                Ok(user) => {
                    let mut posts = db.write().await;
                    
                    match posts.get_mut(&request.post_id) {
                        Some(post) => {
                            // Check if user owns the post
                            if post.author_id != user.id {
                                return Ok(RpcResponse::error("You can only update your own posts".to_string()));
                            }
                            
                            // Update fields if provided
                            if let Some(title) = request.title {
                                post.title = title;
                            }
                            if let Some(content) = request.content {
                                post.content = content;
                            }
                            if let Some(tags) = request.tags {
                                post.tags = tags;
                            }
                            post.updated_at = chrono::Utc::now().to_rfc3339();
                            
                            info!("✅ Post updated successfully: {}", post.title);
                            Ok(RpcResponse::success(post.clone(), 15)?)
                        }
                        None => {
                            Ok(RpcResponse::error("Post not found".to_string()))
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Authentication failed: {}", e);
                    Ok(RpcResponse::error(format!("Authentication failed: {}", e)))
                }
            }
        }
    }).await;

    // Get all posts handler
    let db_clone = db.clone();
    service.register_function("list_posts", move |_msg: Message| {
        let db = db_clone.clone();
        async move {
            info!("📋 Listing all posts");
            
            let posts = db.read().await;
            let post_list: Vec<Post> = posts.values()
                .filter(|p| p.is_published)
                .cloned()
                .collect();
            
            info!("✅ Listed {} published posts", post_list.len());
            Ok(RpcResponse::success(post_list, 10)?)
        }
    }).await;

    // Get post by ID
    let db_clone = db.clone();
    service.register_function("get_post", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            let post_id: String = msg.deserialize_payload()?;
            info!("🔍 Getting post: {}", post_id);
            
            let posts = db.read().await;
            match posts.get(&post_id) {
                Some(post) if post.is_published => {
                    info!("✅ Found post: {}", post.title);
                    Ok(RpcResponse::success(post.clone(), 5)?)
                }
                Some(_) => {
                    Ok(RpcResponse::error("Post not published".to_string()))
                }
                None => {
                    Ok(RpcResponse::error("Post not found".to_string()))
                }
            }
        }
    }).await;

    // Get posts by author
    let db_clone = db.clone();
    service.register_function("get_posts_by_author", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            let request: GetPostsByAuthorRequest = msg.deserialize_payload()?;
            info!("👤 Getting posts by author: {}", request.author_id);
            
            let posts = db.read().await;
            let author_posts: Vec<Post> = posts.values()
                .filter(|p| p.author_id == request.author_id && p.is_published)
                .cloned()
                .collect();
            
            info!("✅ Found {} posts by author", author_posts.len());
            Ok(RpcResponse::success(author_posts, 10)?)
        }
    }).await;

    // Health check
    service.register_function("ping", |_msg: Message| async move {
        Ok(RpcResponse::success("Post service is healthy", 1)?)
    }).await;

    Ok(())
}