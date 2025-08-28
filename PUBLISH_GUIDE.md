# 🚀 Publishing RabbitMesh to Crates.io

Complete guide to publish the RabbitMesh framework to crates.io for real-world usage.

## 📋 Pre-Publishing Checklist

### ✅ Completed
- [x] Updated Cargo.toml with proper metadata (version, authors, description, etc.)
- [x] Added LICENSE-MIT and LICENSE-APACHE files
- [x] Created comprehensive README.md
- [x] Removed workspace dependencies for individual crate publishing
- [x] Added proper keywords and categories for discoverability

### 🔍 Before Publishing

1. **Test the workspace builds correctly**
```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace
```

2. **Check individual crate publishing readiness**
```bash
# Test dry-run publishing (doesn't actually publish)
cd rabbitmesh-macros
cargo publish --dry-run

cd ../rabbitmesh  
cargo publish --dry-run

cd ../rabbitmesh-gateway
cargo publish --dry-run
```

3. **Verify documentation builds**
```bash
cargo doc --no-deps --open
```

## 🚀 Publishing Steps

### Step 1: Login to Crates.io
```bash
# Get API token from https://crates.io/me
cargo login YOUR_API_TOKEN
```

### Step 2: Publish in Dependency Order

**Important**: Publish in the correct order since `rabbitmesh` depends on `rabbitmesh-macros`:

```bash
# 1. First publish the macros (no dependencies)
cd rabbitmesh-macros
cargo publish

# 2. Then publish the main framework (depends on macros)  
cd ../rabbitmesh
cargo publish

# 3. Finally publish the gateway (depends on rabbitmesh)
cd ../rabbitmesh-gateway
cargo publish
```

### Step 3: Verify Publication
- Check https://crates.io/crates/rabbitmesh
- Check https://crates.io/crates/rabbitmesh-macros  
- Check https://crates.io/crates/rabbitmesh-gateway
- Verify documentation at https://docs.rs

## 🧪 Test the Published Crates

### Create a Test Project
```bash
# Create new project to test published crates
cargo new rabbitmesh-test
cd rabbitmesh-test

# Add dependencies
cargo add rabbitmesh@0.1.0
cargo add rabbitmesh-macros@0.1.0  
cargo add serde --features derive
cargo add tokio --features full
cargo add anyhow
```

### Test Service Creation
```rust
// src/main.rs
use rabbitmesh_macros::{service_definition, service_impl};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateItemRequest {
    pub name: String,
}

#[derive(Serialize)]
pub struct ItemResponse {
    pub success: bool,
    pub message: String,
}

#[service_definition]
pub struct TestService;

#[service_impl]
impl TestService {
    #[service_method("POST /items")]
    pub async fn create_item(request: CreateItemRequest) -> Result<ItemResponse, String> {
        Ok(ItemResponse {
            success: true,
            message: format!("Created item: {}", request.name),
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Testing RabbitMesh from crates.io...");
    
    let rabbitmq_url = "amqp://guest:guest@localhost:5672/%2f";
    let service = TestService::create_service(&rabbitmq_url).await?;
    
    println!("✅ Service created successfully!");
    println!("🎯 Service name: {}", TestService::service_name());
    
    let routes = TestService::get_routes();
    println!("📊 Auto-discovered routes:");
    for (route, method) in routes {
        println!("   {} -> {}", method, route);
    }
    
    // Don't actually start (would need RabbitMQ running)
    println!("🎉 RabbitMesh from crates.io works perfectly!");
    Ok(())
}
```

## 🔧 Fixing Common Publishing Issues

### Issue: Workspace Dependencies
**Problem**: `version.workspace = true` doesn't work for publishing

**Fix**: Replace workspace dependencies with explicit versions in each crate's Cargo.toml

### Issue: Missing Metadata
**Problem**: Crates.io requires description, license, etc.

**Fix**: Add all required metadata fields to Cargo.toml

### Issue: Dependency Versions
**Problem**: Path dependencies won't work for published crates

**Fix**: Update inter-crate dependencies to use version numbers:
```toml
[dependencies]
rabbitmesh-macros = "0.1.0"  # Instead of path = "../rabbitmesh-macros"
```

## 📊 Post-Publishing

### 1. Update Project Documentation
- Update README with `cargo add rabbitmesh` instructions
- Update examples to use published crates
- Create Quick Start guide for new users

### 2. Announce Release
- Post on Reddit r/rust
- Tweet about the release
- Write blog post about zero-port microservices

### 3. Monitor and Maintain
- Watch for issues and bug reports
- Keep dependencies up to date
- Plan version updates and new features

## 🎯 Version Update Process

For future updates:

1. **Update version numbers** in all Cargo.toml files
2. **Run tests** to ensure everything works
3. **Publish in dependency order** (macros → rabbitmesh → gateway)
4. **Tag the release** in Git: `git tag v0.1.1`
5. **Update documentation** if needed

## 🔍 Publishing Command Reference

```bash
# Test publishing without actually doing it
cargo publish --dry-run

# Actually publish to crates.io
cargo publish

# Publish specific version
cargo publish --version 0.1.1

# Check what would be included in package
cargo package --list
```

## ⚠️ Important Notes

- **Cannot unpublish**: Once published, versions cannot be deleted from crates.io
- **Version must be unique**: Each version number can only be used once
- **Semver compliance**: Follow semantic versioning for updates
- **Dependencies**: Ensure all dependencies are also published to crates.io

Ready to revolutionize microservices development! 🚀