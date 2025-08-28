use crate::contracts::{ValidateTokenRequest, UserProfile, PostExistsRequest, PostExistsResponse};
use rabbitmesh::ServiceClient;
use std::time::Duration;
use tracing::{info, error};

/// Validates token with auth service and returns user profile
/// This is the shared function used by all services that need auth
pub async fn validate_user_token(
    client: &ServiceClient,
    token: &str,
) -> Result<UserProfile, String> {
    info!("🔐 Validating token with auth service");
    
    let request = ValidateTokenRequest {
        token: token.to_string(),
    };
    
    match client.call_with_timeout(
        "auth-service",
        "validate_token", 
        request,
        Duration::from_secs(10)
    ).await {
        Ok(response) => {
            match response.data::<UserProfile>() {
                Ok(user) => {
                    info!("✅ Token validated for user: {}", user.username);
                    Ok(user)
                }
                Err(e) => {
                    error!("❌ Failed to deserialize auth response: {}", e);
                    Err("Invalid authentication response".to_string())
                }
            }
        }
        Err(e) => {
            error!("❌ Auth service call failed: {}", e);
            Err("Authentication service unavailable".to_string())
        }
    }
}

/// Verifies post exists with post service
/// Returns minimal post info for validation (not full internal model)
pub async fn verify_post_exists(
    client: &ServiceClient,
    post_id: &str,
) -> Result<PostExistsResponse, String> {
    info!("📝 Verifying post exists with post service: {}", post_id);
    
    let request = PostExistsRequest {
        post_id: post_id.to_string(),
    };
    
    match client.call_with_timeout(
        "post-service",
        "post_exists", 
        request,
        Duration::from_secs(10)
    ).await {
        Ok(response) => {
            match response.data::<PostExistsResponse>() {
                Ok(post_info) => {
                    if post_info.exists {
                        info!("✅ Post verified: {}", post_info.title.as_ref().unwrap_or(&"Unknown".to_string()));
                        Ok(post_info)
                    } else {
                        error!("❌ Post not found: {}", post_id);
                        Err("Post not found".to_string())
                    }
                }
                Err(e) => {
                    error!("❌ Failed to deserialize post response: {}", e);
                    Err("Invalid post response".to_string())
                }
            }
        }
        Err(e) => {
            error!("❌ Post service call failed: {}", e);
            Err("Post service unavailable".to_string())
        }
    }
}