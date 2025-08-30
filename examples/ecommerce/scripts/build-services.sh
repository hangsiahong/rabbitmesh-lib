#!/bin/bash

# RabbitMesh E-Commerce - Build Services Script
# This script builds all services in release mode for production deployment

set -e

echo "🔨 Building RabbitMesh E-Commerce Services..."
echo "========================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    print_error "Cargo (Rust) is not installed. Please install Rust first."
    exit 1
fi

# Create logs directory
mkdir -p logs

# Build workspace
print_status "Building entire workspace..."
cargo build --release

if [ $? -eq 0 ]; then
    print_success "Workspace build completed successfully"
else
    print_error "Workspace build failed"
    exit 1
fi

# Copy binaries to respective service directories for PM2
print_status "Copying binaries to service directories..."

services=("user-service" "auth-service" "order-service" "gateway")

for service in "${services[@]}"; do
    if [ -f "target/release/$service" ]; then
        mkdir -p "$service/target/release"
        cp "target/release/$service" "$service/target/release/"
        print_success "Copied $service binary"
    else
        print_error "Binary for $service not found"
        exit 1
    fi
done

# Check if services are properly built
print_status "Verifying service binaries..."

for service in "${services[@]}"; do
    if [ -x "$service/target/release/$service" ]; then
        print_success "$service binary is ready"
    else
        print_error "$service binary is not executable"
        exit 1
    fi
done

echo ""
print_success "🎉 All services built successfully!"
echo ""
print_status "Next steps:"
echo "  1. Start infrastructure: docker-compose up -d rabbitmq mongodb redis"
echo "  2. Start services: pm2 start ecosystem.config.js"
echo "  3. Check status: pm2 status"
echo "  4. View logs: pm2 logs"
echo ""
print_status "API Gateway will be available at: http://localhost:3000"
print_status "RabbitMQ Management UI: http://localhost:15672 (guest/guest)"
print_status "API Documentation: http://localhost:3000/api-docs"