// Example usage of the generated Blog Platform client
import { RabbitMeshClient } from './index';

async function runBlogPlatformExample() {
  // Initialize client - usually you'd get this from environment variables
  const client = new RabbitMeshClient('http://localhost:3000');

  try {
    console.log('🚀 Blog Platform Client Example\n');

    // === AUTHENTICATION FLOW ===
    console.log('🔐 === Authentication Flow ===');

    // Register new user
    console.log('👤 Registering new user...');
    const registerResult = await client.auth.register({
      username: "johndoe",
      email: "john@example.com",
      password: "securepassword",
      full_name: "John Doe"
    });
    console.log('✅ Registration result:', registerResult);

    // Login user and get JWT token
    console.log('🔑 Logging in user...');
    const loginResult = await client.auth.login({
      email: "john@example.com",
      password: "securepassword"
    });
    console.log('✅ Login result:', loginResult);

    if (!loginResult.success || !loginResult.token) {
      throw new Error('Login failed');
    }

    const jwtToken = loginResult.token;
    const userId = loginResult.user?.id;

    console.log('🎯 JWT Token obtained:', jwtToken.substring(0, 50) + '...');

    // === BLOG POST MANAGEMENT ===
    console.log('\n📝 === Blog Post Management ===');

    // Create a blog post (requires authentication)
    console.log('📄 Creating new blog post...');
    const createPostResult = await client.blog.createPost(
      `Bearer ${jwtToken}`, // Authorization header
      {
        title: "My First Blog Post",
        content: "This is the content of my first blog post. It talks about how amazing RabbitMesh is for building microservices with TypeScript clients that have full autocomplete support!",
        tags: ["tech", "programming", "microservices"],
        status: "published" as any
      }
    );
    console.log('✅ Blog post created:', createPostResult);

    if (!createPostResult.success || !createPostResult.post) {
      throw new Error('Post creation failed');
    }

    const postId = createPostResult.post.id;

    // Get the blog post we just created (no auth required for reading)
    console.log('\n🔍 Fetching the blog post...');
    const fetchedPost = await client.blog.getPost(postId);
    console.log('✅ Fetched post:', fetchedPost);

    // Update the blog post (requires authentication and ownership)
    console.log('\n✏️ Updating the blog post...');
    const updateResult = await client.blog.updatePost(
      `Bearer ${jwtToken}`,
      postId,
      {
        content: "Updated content! This post now has even more amazing information about RabbitMesh.",
        tags: ["tech", "programming", "microservices", "updated"]
      }
    );
    console.log('✅ Post updated:', updateResult);

    // === BLOG LISTING ===
    console.log('\n📋 === Blog Post Listing ===');

    // List all published posts (no auth required)
    console.log('📜 Listing all blog posts...');
    const allPosts = await client.blog.listPosts(1, 10, "published");
    console.log(`✅ Found ${allPosts.total} posts:`, allPosts.posts.map(p => ({
      id: p.id,
      title: p.title,
      author: p.author_name,
      status: p.status
    })));

    // === COMMENTS SYSTEM ===
    console.log('\n💬 === Comments System ===');

    // Create a comment on the blog post (requires authentication)
    console.log('💭 Creating comment on blog post...');
    const commentResult = await client.blog.createComment(
      `Bearer ${jwtToken}`,
      postId,
      {
        post_id: postId,
        content: "Great post! RabbitMesh really does make microservices development much easier.",
        parent_id: null // Top-level comment
      }
    );
    console.log('✅ Comment created:', commentResult);

    // Create a nested reply comment
    if (commentResult.success && commentResult.comment) {
      console.log('↪️ Creating reply to comment...');
      const replyResult = await client.blog.createComment(
        `Bearer ${jwtToken}`,
        postId,
        {
          post_id: postId,
          content: "Thanks! I'm glad you found it helpful.",
          parent_id: commentResult.comment.id // Reply to the first comment
        }
      );
      console.log('✅ Reply created:', replyResult);
    }

    // Get all comments for the post
    console.log('📄 Fetching all comments for the post...');
    const comments = await client.blog.getComments(postId);
    console.log(`✅ Found ${comments.total} comments:`, comments.comments.map(c => ({
      id: c.id,
      author: c.author_name,
      content: c.content.substring(0, 50) + '...',
      parent_id: c.parent_id
    })));

    // === USER PROFILE MANAGEMENT ===
    console.log('\n👤 === User Profile Management ===');

    if (userId) {
      // Get user profile
      console.log('👨‍💼 Fetching user profile...');
      const profile = await client.auth.getProfile(userId);
      console.log('✅ User profile:', profile);

      // Update user profile
      console.log('✏️ Updating user profile...');
      const updateProfileResult = await client.auth.updateProfile(userId, {
        full_name: "John Doe (Updated)",
        avatar_url: "https://example.com/avatar.jpg"
      });
      console.log('✅ Profile updated:', updateProfileResult);
    }

    // === SERVICE-TO-SERVICE VALIDATION ===
    console.log('\n🔍 === Token Validation (Service-to-Service) ===');

    // This would typically be called by other services to validate tokens
    console.log('🔐 Validating JWT token...');
    const tokenValidation = await client.auth.validateToken({
      token: jwtToken
    });
    console.log('✅ Token validation result:', tokenValidation);

    console.log('\n🎉 === Blog Platform Example Completed Successfully ===');
    console.log('📊 Summary:');
    console.log(`   👤 Users: Registered and logged in`);
    console.log(`   📝 Posts: Created, updated, and listed`);
    console.log(`   💬 Comments: Created top-level and nested comments`);
    console.log(`   🔐 Auth: Token validation working`);
    console.log(`   ⚙️  Services: Auth ↔ Blog service communication demonstrated`);

  } catch (error: any) {
    console.error('❌ Error in blog platform example:', error.response?.data || error.message);
  }
}

// Run the example
if (require.main === module) {
  runBlogPlatformExample().catch(console.error);
}

export { runBlogPlatformExample };