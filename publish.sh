#!/bin/bash

# RabbitMesh Publishing Script
# This script publishes the crates in the correct dependency order

set -e

echo "🚀 RabbitMesh Publishing Script"
echo "================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}📋 Publishing Order:${NC}"
echo "1. rabbitmesh-macros (no RabbitMesh dependencies)"
echo "2. rabbitmesh (depends on rabbitmesh-macros)"
echo "3. rabbitmesh-gateway (depends on rabbitmesh + rabbitmesh-macros)"
echo ""

read -p "Continue with publishing? (y/N): " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 1
fi

echo ""
echo -e "${YELLOW}Step 1: Publishing rabbitmesh-macros${NC}"
echo "======================================"
cd rabbitmesh-macros
if cargo publish; then
    echo -e "${GREEN}✅ rabbitmesh-macros published successfully${NC}"
else
    echo -e "${RED}❌ Failed to publish rabbitmesh-macros${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}Step 2: Updating rabbitmesh dependencies and publishing${NC}"
echo "======================================================"
cd ../rabbitmesh

# Update to use published version
sed -i '' 's|rabbitmesh-macros = { path = "../rabbitmesh-macros" }|rabbitmesh-macros = "0.1.0"|g' Cargo.toml

if cargo publish; then
    echo -e "${GREEN}✅ rabbitmesh published successfully${NC}"
    # Revert to path dependency for development
    sed -i '' 's|rabbitmesh-macros = "0.1.0"|rabbitmesh-macros = { path = "../rabbitmesh-macros" }|g' Cargo.toml
else
    echo -e "${RED}❌ Failed to publish rabbitmesh${NC}"
    # Revert to path dependency
    sed -i '' 's|rabbitmesh-macros = "0.1.0"|rabbitmesh-macros = { path = "../rabbitmesh-macros" }|g' Cargo.toml
    exit 1
fi

echo ""
echo -e "${YELLOW}Step 3: Updating rabbitmesh-gateway dependencies and publishing${NC}"
echo "=============================================================="
cd ../rabbitmesh-gateway

# Update to use published versions
sed -i '' 's|rabbitmesh = { path = "../rabbitmesh" }|rabbitmesh = "0.1.0"|g' Cargo.toml
sed -i '' 's|rabbitmesh-macros = { path = "../rabbitmesh-macros" }|rabbitmesh-macros = "0.1.0"|g' Cargo.toml

if cargo publish; then
    echo -e "${GREEN}✅ rabbitmesh-gateway published successfully${NC}"
    # Revert to path dependencies for development
    sed -i '' 's|rabbitmesh = "0.1.0"|rabbitmesh = { path = "../rabbitmesh" }|g' Cargo.toml
    sed -i '' 's|rabbitmesh-macros = "0.1.0"|rabbitmesh-macros = { path = "../rabbitmesh-macros" }|g' Cargo.toml
else
    echo -e "${RED}❌ Failed to publish rabbitmesh-gateway${NC}"
    # Revert to path dependencies
    sed -i '' 's|rabbitmesh = "0.1.0"|rabbitmesh = { path = "../rabbitmesh" }|g' Cargo.toml
    sed -i '' 's|rabbitmesh-macros = "0.1.0"|rabbitmesh-macros = { path = "../rabbitmesh-macros" }|g' Cargo.toml
    exit 1
fi

cd ..

echo ""
echo -e "${GREEN}🎉 All RabbitMesh crates published successfully!${NC}"
echo ""
echo -e "${BLUE}📦 Published crates:${NC}"
echo "  • rabbitmesh-macros v0.1.0"
echo "  • rabbitmesh v0.1.0" 
echo "  • rabbitmesh-gateway v0.1.0"
echo ""
echo -e "${BLUE}🚀 Users can now add to their Cargo.toml:${NC}"
echo "rabbitmesh = \"0.1.0\""
echo "rabbitmesh-macros = \"0.1.0\""
echo "rabbitmesh-gateway = \"0.1.0\""
echo ""
echo -e "${GREEN}Ready for real-world usage! 🪄${NC}"