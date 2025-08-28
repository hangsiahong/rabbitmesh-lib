use uuid::Uuid;
pub use blog_common::validate_user_token;

pub fn generate_post_id() -> String {
    Uuid::new_v4().to_string()
}