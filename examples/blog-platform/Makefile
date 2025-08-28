# RabbitMesh Blog Platform Makefile
# Makes it easy to start/stop all microservices

.PHONY: help start stop status logs clean build check deps rabbitmq rabbitmq-stop restart

# Default target
help: ## Show this help message
	@echo "🚀 RabbitMesh Blog Platform - Service Management"
	@echo ""
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Example workflow:"
	@echo "  make deps          # Install dependencies"  
	@echo "  make rabbitmq      # Start RabbitMQ"
	@echo "  make start         # Start all services"
	@echo "  make logs          # View service logs"
	@echo "  make stop          # Stop all services"

# Dependency management
deps: ## Install and check dependencies
	@echo "📦 Checking dependencies..."
	@which docker >/dev/null || (echo "❌ Docker not found. Please install Docker first." && exit 1)
	@echo "✅ Docker found"
	@which cargo >/dev/null || (echo "❌ Cargo not found. Please install Rust first." && exit 1)  
	@echo "✅ Cargo found"
	@echo "🔧 Building all services..."
	@cargo build
	@echo "✅ All dependencies ready!"

# RabbitMQ management
rabbitmq: ## Start RabbitMQ in Docker
	@echo "🐰 Starting RabbitMQ..."
	@if docker ps --format "{{.Ports}}" | grep -q "5672->5672"; then \
		echo "✅ RabbitMQ already running on port 5672"; \
	else \
		docker run -d \
			--name rabbitmq-blog \
			-p 5672:5672 \
			-p 15672:15672 \
			rabbitmq:3-management; \
		echo "✅ RabbitMQ started"; \
	fi
	@echo "📋 AMQP: localhost:5672, Management UI: http://localhost:15672 (guest/guest)"

rabbitmq-stop: ## Stop RabbitMQ Docker container
	@echo "🛑 Stopping RabbitMQ..."
	@docker stop rabbitmq-blog 2>/dev/null || true
	@docker rm rabbitmq-blog 2>/dev/null || true
	@echo "✅ RabbitMQ stopped"

rabbitmq-status: ## Check RabbitMQ status
	@echo "🐰 RabbitMQ Status:"
	@docker ps --filter "publish=5672" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" 2>/dev/null | head -n 10 || echo "❌ No RabbitMQ container found on port 5672"

# Service management
start: rabbitmq ## Start all microservices
	@echo "🚀 Starting Blog Platform Services..."
	@echo ""
	@echo "📝 Starting services in background..."
	@mkdir -p logs
	@cargo run --bin auth-service > logs/auth-service.log 2>&1 & echo $$! > logs/auth-service.pid
	@sleep 2
	@cargo run --bin post-service > logs/post-service.log 2>&1 & echo $$! > logs/post-service.pid  
	@sleep 2
	@cargo run --bin comment-service > logs/comment-service.log 2>&1 & echo $$! > logs/comment-service.pid
	@sleep 2
	@cargo run --bin blog-api-gateway > logs/api-gateway.log 2>&1 & echo $$! > logs/api-gateway.pid
	@sleep 3
	@echo ""
	@echo "✅ All services started!"
	@echo ""
	@echo "🌐 API Gateway: http://localhost:3334"
	@echo "📋 API Docs: http://localhost:3334/docs"  
	@echo "🔗 OpenAPI Spec: http://localhost:3334/api-docs/openapi.json"
	@echo "🐰 RabbitMQ UI: http://localhost:15672 (guest/guest)"
	@echo ""
	@echo "📊 Use 'make status' to check service health"
	@echo "📜 Use 'make logs' to view service logs"

stop: ## Stop all microservices
	@echo "🛑 Stopping Blog Platform Services..."
	@if [ -f logs/auth-service.pid ]; then kill -TERM $$(cat logs/auth-service.pid) 2>/dev/null || true; rm -f logs/auth-service.pid; fi
	@if [ -f logs/post-service.pid ]; then kill -TERM $$(cat logs/post-service.pid) 2>/dev/null || true; rm -f logs/post-service.pid; fi
	@if [ -f logs/comment-service.pid ]; then kill -TERM $$(cat logs/comment-service.pid) 2>/dev/null || true; rm -f logs/comment-service.pid; fi
	@if [ -f logs/api-gateway.pid ]; then kill -TERM $$(cat logs/api-gateway.pid) 2>/dev/null || true; rm -f logs/api-gateway.pid; fi
	@sleep 2
	@pkill -f "cargo run --bin.*-service" 2>/dev/null || true
	@pkill -f "blog-api-gateway" 2>/dev/null || true
	@echo "✅ All services stopped"

restart: stop start ## Restart all services

status: ## Check status of all services
	@echo "📊 Blog Platform Status:"
	@echo ""
	@echo "🐰 RabbitMQ:"
	@docker ps --filter "publish=5672" --format "  ✅ {{.Names}} - {{.Status}}" 2>/dev/null | head -n 1 || echo "  ❌ RabbitMQ not running on port 5672"
	@echo ""
	@echo "🔧 Microservices:"
	@if [ -f logs/auth-service.pid ] && kill -0 $$(cat logs/auth-service.pid) 2>/dev/null; then echo "  ✅ Auth Service - Running (PID: $$(cat logs/auth-service.pid))"; else echo "  ❌ Auth Service - Not running"; fi
	@if [ -f logs/post-service.pid ] && kill -0 $$(cat logs/post-service.pid) 2>/dev/null; then echo "  ✅ Post Service - Running (PID: $$(cat logs/post-service.pid))"; else echo "  ❌ Post Service - Not running"; fi
	@if [ -f logs/comment-service.pid ] && kill -0 $$(cat logs/comment-service.pid) 2>/dev/null; then echo "  ✅ Comment Service - Running (PID: $$(cat logs/comment-service.pid))"; else echo "  ❌ Comment Service - Not running"; fi  
	@if [ -f logs/api-gateway.pid ] && kill -0 $$(cat logs/api-gateway.pid) 2>/dev/null; then echo "  ✅ API Gateway - Running (PID: $$(cat logs/api-gateway.pid))"; else echo "  ❌ API Gateway - Not running"; fi
	@echo ""
	@echo "🌐 Service Health Checks:"
	@curl -s http://localhost:3334/health >/dev/null 2>&1 && echo "  ✅ Gateway Health - OK" || echo "  ❌ Gateway Health - Failed"
	@curl -s http://localhost:3334/health/auth-service >/dev/null 2>&1 && echo "  ✅ Auth Service Health - OK" || echo "  ❌ Auth Service Health - Failed"
	@curl -s http://localhost:3334/health/post-service >/dev/null 2>&1 && echo "  ✅ Post Service Health - OK" || echo "  ❌ Post Service Health - Failed"
	@curl -s http://localhost:3334/health/comment-service >/dev/null 2>&1 && echo "  ✅ Comment Service Health - OK" || echo "  ❌ Comment Service Health - Failed"

logs: ## View logs from all services
	@echo "📜 Service Logs (Ctrl+C to exit):"
	@echo ""
	@if [ -d logs ]; then \
		echo "🔐 Auth Service Logs:"; echo "================"; tail -n 10 logs/auth-service.log 2>/dev/null || echo "No logs found"; echo ""; \
		echo "📝 Post Service Logs:"; echo "================"; tail -n 10 logs/post-service.log 2>/dev/null || echo "No logs found"; echo ""; \
		echo "💬 Comment Service Logs:"; echo "=================="; tail -n 10 logs/comment-service.log 2>/dev/null || echo "No logs found"; echo ""; \
		echo "🌐 API Gateway Logs:"; echo "==============="; tail -n 10 logs/api-gateway.log 2>/dev/null || echo "No logs found"; echo ""; \
		echo "📡 Follow live logs with:"; \
		echo "  tail -f logs/auth-service.log"; \
		echo "  tail -f logs/post-service.log"; \
		echo "  tail -f logs/comment-service.log"; \
		echo "  tail -f logs/api-gateway.log"; \
	else \
		echo "❌ No logs directory found. Start services first with 'make start'"; \
	fi

logs-follow: ## Follow live logs from all services
	@echo "📡 Following live logs (Ctrl+C to exit):"
	@if [ -d logs ]; then \
		tail -f logs/*.log; \
	else \
		echo "❌ No logs directory found. Start services first with 'make start'"; \
	fi

# Development helpers  
build: ## Build all services
	@echo "🔧 Building all services..."
	@cargo build
	@echo "✅ Build complete"

check: ## Run tests and checks
	@echo "🔍 Running tests and checks..."
	@cargo test
	@cargo clippy
	@echo "✅ All checks passed"

clean: ## Clean build artifacts and logs
	@echo "🧹 Cleaning up..."
	@cargo clean
	@rm -rf logs
	@echo "✅ Cleanup complete"

# Quick test commands
test-api: ## Quick API test (requires services to be running)
	@echo "🧪 Testing API endpoints..."
	@echo "1. Testing health endpoints:"
	@curl -s http://localhost:3334/health | jq . || echo "❌ Gateway health failed"
	@curl -s http://localhost:3334/health/auth-service | jq . || echo "❌ Auth service health failed"
	@echo ""
	@echo "2. Testing user registration:"
	@curl -s -X POST http://localhost:3334/api/v1/auth-service/register \
		-H 'Content-Type: application/json' \
		-d '{"username":"testuser","email":"test@example.com","password":"test123"}' | jq . || echo "❌ User registration failed"
	@echo ""
	@echo "3. Testing posts list:"
	@curl -s http://localhost:3334/api/v1/post-service/list_posts | jq . || echo "❌ Posts list failed"
	@echo ""
	@echo "✅ API test complete"

# Development workflow
dev: ## Start development environment
	@echo "🔧 Starting development environment..."
	@make deps
	@make rabbitmq
	@sleep 3
	@make start
	@echo ""
	@echo "🎉 Development environment ready!"
	@echo "📋 Open API docs: http://localhost:3334/docs"

# Production-like commands
production-stop: rabbitmq-stop stop clean ## Full cleanup for production

info: ## Show useful information
	@echo "ℹ️  RabbitMesh Blog Platform Information"
	@echo ""
	@echo "🌐 Endpoints:"
	@echo "  API Gateway:     http://localhost:3334"
	@echo "  API Docs:        http://localhost:3334/docs"
	@echo "  OpenAPI Spec:    http://localhost:3334/api-docs/openapi.json"
	@echo "  RabbitMQ UI:     http://localhost:15672 (guest/guest)"
	@echo ""
	@echo "🔗 Service Architecture:"
	@echo "  Auth Service:    Zero ports (RabbitMQ only)"
	@echo "  Post Service:    Zero ports (RabbitMQ only)"
	@echo "  Comment Service: Zero ports (RabbitMQ only)"
	@echo "  API Gateway:     Port 3334 (Only HTTP port in system)"
	@echo ""
	@echo "💬 Inter-Service Communication:"
	@echo "  Comment → Auth (token validation)"
	@echo "  Comment → Post (post verification)"
	@echo "  Post → Auth (user authentication)"
	@echo ""
	@echo "📁 File Structure:"
	@echo "  blog-common/     - Shared utilities & models"  
	@echo "  auth-service/    - User management"
	@echo "  post-service/    - Blog posts"
	@echo "  comment-service/ - Comments with multi-service calls"
	@echo "  api-gateway/     - HTTP gateway with OpenAPI docs"