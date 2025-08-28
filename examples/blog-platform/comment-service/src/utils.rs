use uuid::Uuid;
pub use blog_common::{validate_user_token, verify_post_exists};

pub fn generate_comment_id() -> String {
    Uuid::new_v4().to_string()
}