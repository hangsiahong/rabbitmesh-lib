use utoipa::OpenApi;
use crate::contracts::*;

#[derive(OpenApi)]
#[openapi(
    paths(),
    components(schemas(
        // Auth contract models
        UserProfile,
        ValidateTokenRequest,
        ValidateTokenResponse,
        GetUserRequest,
        
        // Post contract models
        PostExistsRequest,
        PostExistsResponse,
        PostReference,
        
        // Comment contract models
        CommentReference,
    )),
    tags(
        (name = "auth", description = "Authentication and user management"),
        (name = "posts", description = "Blog post management"),
        (name = "comments", description = "Comment management with multi-service integration"),
        (name = "health", description = "Health check endpoints"),
    ),
    info(
        title = "RabbitMesh Blog Platform API",
        version = "1.0.0",
        description = "A revolutionary microservices blog platform demonstrating service-to-service communication via RabbitMQ",
        contact(
            name = "RabbitMesh Team",
            email = "info@rabbitmesh.dev"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:3334", description = "Local development server")
    )
)]
pub struct ApiDoc;

pub fn get_openapi_spec() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}