# Database Macros

Transaction management, connection pooling, query optimization, and database migration support for robust data persistence in microservices.

## Overview

Database macros provide comprehensive database integration with automatic transaction management, connection pooling, query optimization, and migration support across multiple database backends.

---

## Transaction Management Macros

### `#[transactional]`

**Purpose**: Wraps method execution in a database transaction with automatic rollback on errors.

**Usage**:
```rust
#[service_method("POST /orders")]
#[require_auth]
#[validate]
#[transactional]
pub async fn create_order(msg: Message) -> Result<RpcResponse, String> {
    let order_data: CreateOrderRequest = msg.deserialize_payload()?;
    let user_id = msg.get_user_id()?;
    
    // All database operations within this method are automatically wrapped
    // in a transaction. If any operation fails, the entire transaction
    // is rolled back.
    
    // 1. Create the order
    let order = insert_order(&order_data, &user_id).await?;
    
    // 2. Reserve inventory
    for item in &order_data.items {
        reserve_inventory_item(&item.product_id, item.quantity).await?;
    }
    
    // 3. Create audit log entry
    create_audit_log_entry("order_created", &order.id, &user_id).await?;
    
    // 4. Send notification (this might fail, but transaction should still commit)
    if let Err(e) = send_order_confirmation_email(&user_id, &order.id).await {
        tracing::warn!("Failed to send confirmation email: {}", e);
        // Don't fail the transaction for notification failures
    }
    
    // Transaction automatically commits if method returns Ok
    Ok(RpcResponse::success(&order, 0)?)
}

#[service_method("PUT /orders/:id/status")]
#[require_auth]
#[require_permission("orders:update")]
#[transactional(isolation = "ReadCommitted")]
pub async fn update_order_status(msg: Message) -> Result<RpcResponse, String> {
    let order_id = msg.get_path_param("id")?;
    let status_update: OrderStatusUpdate = msg.deserialize_payload()?;
    
    // Transaction with specific isolation level
    let updated_order = update_order_status_in_db(&order_id, &status_update).await?;
    log_status_change(&order_id, &status_update).await?;
    
    Ok(RpcResponse::success(&updated_order, 0)?)
}
```

**Transaction Features**:
- **Automatic Rollback**: Roll back on any error
- **Isolation Levels**: ReadUncommitted, ReadCommitted, RepeatableRead, Serializable
- **Nested Transactions**: Support for savepoints
- **Connection Management**: Automatic connection acquisition and release
- **Deadlock Detection**: Automatic deadlock detection and retry

**Isolation Levels**:
- **ReadUncommitted**: Fastest, allows dirty reads
- **ReadCommitted**: Prevents dirty reads (default)
- **RepeatableRead**: Prevents non-repeatable reads
- **Serializable**: Strongest, prevents phantom reads

**When to Use**: For operations that must be atomic across multiple database operations.

---

### `#[transactional(propagation)]`

**Purpose**: Controls how transactions propagate when calling other transactional methods.

**Usage**:
```rust
#[transactional] // Starts new transaction
pub async fn process_bulk_orders(orders: Vec<CreateOrderRequest>) -> Result<Vec<Order>, OrderError> {
    let mut processed_orders = Vec::new();
    
    for order_request in orders {
        // Each order processed in separate transaction
        let order = process_single_order(order_request).await?;
        processed_orders.push(order);
    }
    
    Ok(processed_orders)
}

#[transactional(propagation = "Required")] // Join existing or create new
pub async fn process_single_order(order_request: CreateOrderRequest) -> Result<Order, OrderError> {
    let order = create_order_record(&order_request).await?;
    process_order_items(&order.id, &order_request.items).await?;
    Ok(order)
}

#[transactional(propagation = "RequiresNew")] // Always start new transaction
pub async fn audit_log_operation(operation: &str, entity_id: &str) -> Result<(), AuditError> {
    // This always runs in its own transaction, even if called from another
    // transactional method. Useful for audit logging that should persist
    // even if the parent transaction fails.
    
    insert_audit_record(operation, entity_id).await?;
    Ok(())
}

#[transactional(propagation = "NotSupported")] // Run without transaction
pub async fn send_notifications(user_id: &str, message: &str) -> Result<(), NotificationError> {
    // This runs outside of any transaction context
    // Useful for operations that shouldn't be part of the main transaction
    
    send_email_notification(user_id, message).await?;
    Ok(())
}
```

**Propagation Types**:
- **Required**: Join existing transaction or create new (default)
- **RequiresNew**: Always create new transaction
- **Supports**: Join existing transaction if present, otherwise run without transaction
- **NotSupported**: Always run without transaction
- **Never**: Fail if called within a transaction
- **Mandatory**: Require existing transaction, fail if none

**When to Use**: For fine-grained control over transaction boundaries in complex operations.

---

### `#[read_only_transaction]`

**Purpose**: Optimized read-only transactions for better performance and resource utilization.

**Usage**:
```rust
#[service_method("GET /reports/sales")]
#[require_auth]
#[require_role("manager")]
#[read_only_transaction]
pub async fn generate_sales_report(msg: Message) -> Result<RpcResponse, String> {
    let from_date = msg.get_query_param("from")?;
    let to_date = msg.get_query_param("to")?;
    
    // Read-only transaction optimizations:
    // - No lock escalation
    // - Better query plan caching
    // - Reduced resource usage
    // - Can use read replicas
    
    let sales_data = query_sales_data(&from_date, &to_date).await?;
    let analytics = calculate_sales_analytics(&sales_data).await?;
    let report = generate_report_document(&analytics).await?;
    
    Ok(RpcResponse::success(&report, 0)?)
}

#[service_method("GET /orders/:id/details")]
#[require_auth]
#[read_only_transaction(replica = "preferred")]
pub async fn get_order_details(msg: Message) -> Result<RpcResponse, String> {
    let order_id = msg.get_path_param("id")?;
    
    // Prefer read replica for better performance
    let order = fetch_order_with_items(&order_id).await?;
    let customer = fetch_customer_details(&order.customer_id).await?;
    let shipping = fetch_shipping_details(&order.id).await?;
    
    let order_details = OrderDetails {
        order,
        customer,
        shipping,
    };
    
    Ok(RpcResponse::success(&order_details, 0)?)
}
```

**Read-Only Transaction Features**:
- **Performance Optimization**: Optimized for read workloads
- **Replica Routing**: Automatic routing to read replicas
- **Reduced Locking**: Minimal lock acquisition
- **Better Caching**: Improved query plan caching

**When to Use**: For complex read operations that benefit from transaction consistency without write operations.

---

## Connection Pool Macros

### `#[connection_pool]`

**Purpose**: Configures database connection pooling with automatic connection management.

**Usage**:
```rust
#[connection_pool(
    database = "postgresql://user:pass@localhost/orders",
    min_connections = 5,
    max_connections = 20,
    max_lifetime = "30m",
    idle_timeout = "5m",
    connection_timeout = "10s"
)]
pub struct OrderDatabase {
    pool: Arc<PgPool>,
}

#[database_impl]
impl OrderDatabase {
    #[query]
    pub async fn find_order_by_id(&self, order_id: &str) -> Result<Order, DatabaseError> {
        let row = self.pool
            .query_one("SELECT * FROM orders WHERE id = $1", &[&order_id])
            .await?;
            
        Ok(Order::from_row(row)?)
    }
    
    #[query_many]
    pub async fn find_orders_by_customer(&self, customer_id: &str) -> Result<Vec<Order>, DatabaseError> {
        let rows = self.pool
            .query(
                "SELECT * FROM orders WHERE customer_id = $1 ORDER BY created_at DESC",
                &[&customer_id]
            )
            .await?;
            
        let orders = rows.into_iter()
            .map(Order::from_row)
            .collect::<Result<Vec<_>, _>>()?;
            
        Ok(orders)
    }
    
    #[execute]
    pub async fn update_order_status(
        &self,
        order_id: &str,
        status: OrderStatus
    ) -> Result<u64, DatabaseError> {
        let affected = self.pool
            .execute(
                "UPDATE orders SET status = $1, updated_at = NOW() WHERE id = $2",
                &[&status.to_string(), &order_id]
            )
            .await?;
            
        Ok(affected)
    }
    
    #[health_check]
    pub async fn check_database_health(&self) -> Result<DatabaseHealth, DatabaseError> {
        let pool_status = self.pool.status();
        
        // Test connection with simple query
        self.pool
            .query_one("SELECT 1 as test", &[])
            .await?;
            
        Ok(DatabaseHealth {
            connected: true,
            pool_size: pool_status.size,
            idle_connections: pool_status.idle,
            active_connections: pool_status.size - pool_status.idle,
        })
    }
}
```

**Connection Pool Configuration**:
- **min_connections**: Minimum pool size
- **max_connections**: Maximum pool size  
- **max_lifetime**: Maximum connection lifetime
- **idle_timeout**: Idle connection timeout
- **connection_timeout**: Connection acquisition timeout

**Pool Monitoring**:
- Active/idle connection counts
- Connection acquisition time
- Connection lifetime tracking
- Pool exhaustion alerts

**When to Use**: For efficient database connection management in high-throughput services.

---

### `#[database_routing]`

**Purpose**: Routes database operations to appropriate database instances (primary/replica, sharding).

**Usage**:
```rust
#[database_routing]
#[primary_db("postgresql://user:pass@primary:5432/orders")]
#[replica_db("postgresql://user:pass@replica1:5432/orders")]
#[replica_db("postgresql://user:pass@replica2:5432/orders")]
pub struct OrderDatabaseCluster {
    primary: Arc<PgPool>,
    replicas: Vec<Arc<PgPool>>,
}

#[database_impl]
impl OrderDatabaseCluster {
    #[route_to("primary")]
    #[transactional]
    pub async fn create_order(&self, order: &CreateOrderRequest) -> Result<Order, DatabaseError> {
        // Always routes to primary for writes
        let order_id = generate_order_id();
        
        self.primary
            .execute(
                r#"
                INSERT INTO orders (id, customer_id, status, created_at) 
                VALUES ($1, $2, $3, NOW())
                "#,
                &[&order_id, &order.customer_id, &"pending"]
            )
            .await?;
            
        Ok(Order {
            id: order_id,
            customer_id: order.customer_id.clone(),
            status: OrderStatus::Pending,
            created_at: Utc::now(),
        })
    }
    
    #[route_to("replica")]
    #[read_only_transaction]
    pub async fn find_order_by_id(&self, order_id: &str) -> Result<Order, DatabaseError> {
        // Routes to random replica for reads
        let row = self.get_replica()
            .query_one("SELECT * FROM orders WHERE id = $1", &[&order_id])
            .await?;
            
        Ok(Order::from_row(row)?)
    }
    
    #[route_to("primary")] // Force primary for consistent reads
    pub async fn get_order_for_update(&self, order_id: &str) -> Result<Order, DatabaseError> {
        // Use primary when you need consistent reads for subsequent updates
        let row = self.primary
            .query_one("SELECT * FROM orders WHERE id = $1", &[&order_id])
            .await?;
            
        Ok(Order::from_row(row)?)
    }
    
    #[route_by_key("customer_id")] // Shard routing
    pub async fn find_orders_by_customer(&self, customer_id: &str) -> Result<Vec<Order>, DatabaseError> {
        // Routes to shard based on customer_id hash
        let shard = self.get_shard_for_key(customer_id);
        
        let rows = shard
            .query(
                "SELECT * FROM orders WHERE customer_id = $1",
                &[&customer_id]
            )
            .await?;
            
        let orders = rows.into_iter()
            .map(Order::from_row)
            .collect::<Result<Vec<_>, _>>()?;
            
        Ok(orders)
    }
}
```

**Routing Strategies**:
- **Primary/Replica**: Write to primary, read from replicas
- **Consistent Reads**: Force primary for read-after-write consistency
- **Load Balancing**: Distribute reads across replicas
- **Sharding**: Route by key to appropriate shard

**When to Use**: For scaled database architectures with multiple database instances.

---

## Query Optimization Macros

### `#[query_cache]`

**Purpose**: Caches database query results for improved performance.

**Usage**:
```rust
#[database_impl]
impl ProductDatabase {
    #[query]
    #[query_cache(ttl = "5m", key = "product:{id}")]
    pub async fn find_product_by_id(&self, id: &str) -> Result<Product, DatabaseError> {
        let row = self.pool
            .query_one("SELECT * FROM products WHERE id = $1", &[&id])
            .await?;
            
        Ok(Product::from_row(row)?)
    }
    
    #[query_many]
    #[query_cache(ttl = "10m", key = "products:category:{category}")]
    pub async fn find_products_by_category(&self, category: &str) -> Result<Vec<Product>, DatabaseError> {
        let rows = self.pool
            .query(
                "SELECT * FROM products WHERE category = $1 ORDER BY name",
                &[&category]
            )
            .await?;
            
        let products = rows.into_iter()
            .map(Product::from_row)
            .collect::<Result<Vec<_>, _>>()?;
            
        Ok(products)
    }
    
    #[query]
    #[query_cache(
        ttl = "1h", 
        key = "popular_products:{limit}",
        invalidate_on = ["product_update", "inventory_change"]
    )]
    pub async fn get_popular_products(&self, limit: i32) -> Result<Vec<Product>, DatabaseError> {
        let rows = self.pool
            .query(
                r#"
                SELECT p.* FROM products p 
                JOIN product_stats ps ON p.id = ps.product_id 
                ORDER BY ps.popularity_score DESC 
                LIMIT $1
                "#,
                &[&limit]
            )
            .await?;
            
        let products = rows.into_iter()
            .map(Product::from_row)
            .collect::<Result<Vec<_>, _>>()?;
            
        Ok(products)
    }
}
```

**Query Cache Features**:
- **Automatic Key Generation**: Based on method name and parameters
- **TTL Support**: Time-based cache expiration
- **Invalidation**: Invalidate on specific events
- **Conditional Caching**: Cache based on query results

**When to Use**: For expensive queries that are called frequently with similar parameters.

---

### `#[prepared_statement]`

**Purpose**: Uses prepared statements for better performance and security.

**Usage**:
```rust
#[database_impl]
impl OrderDatabase {
    #[prepared_statement(
        name = "find_order_by_id",
        query = "SELECT * FROM orders WHERE id = $1"
    )]
    pub async fn find_order_by_id(&self, order_id: &str) -> Result<Order, DatabaseError> {
        // Uses prepared statement for better performance
        // Statement prepared once and reused
        let row = self.pool
            .query_one_prepared("find_order_by_id", &[&order_id])
            .await?;
            
        Ok(Order::from_row(row)?)
    }
    
    #[prepared_statement(
        name = "update_order_status",
        query = "UPDATE orders SET status = $1, updated_at = NOW() WHERE id = $2"
    )]
    pub async fn update_order_status(
        &self, 
        order_id: &str, 
        status: OrderStatus
    ) -> Result<u64, DatabaseError> {
        let affected = self.pool
            .execute_prepared("update_order_status", &[&status.to_string(), &order_id])
            .await?;
            
        Ok(affected)
    }
    
    #[prepared_statement(
        name = "find_orders_by_date_range",
        query = r#"
            SELECT * FROM orders 
            WHERE created_at >= $1 AND created_at <= $2 
            ORDER BY created_at DESC
        "#
    )]
    pub async fn find_orders_by_date_range(
        &self,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>
    ) -> Result<Vec<Order>, DatabaseError> {
        let rows = self.pool
            .query_prepared("find_orders_by_date_range", &[&from_date, &to_date])
            .await?;
            
        let orders = rows.into_iter()
            .map(Order::from_row)
            .collect::<Result<Vec<_>, _>>()?;
            
        Ok(orders)
    }
}
```

**Prepared Statement Benefits**:
- **Performance**: Query parsing done once
- **Security**: Protection against SQL injection
- **Resource Usage**: Reduced CPU for query parsing
- **Plan Caching**: Better query plan caching

**When to Use**: For frequently executed queries with parameters.

---

## Database Migration Macros

### `#[migration]`

**Purpose**: Defines database migrations with automatic versioning and dependency management.

**Usage**:
```rust
#[migration(version = 1, description = "Create orders table")]
pub async fn create_orders_table(db: &DatabaseConnection) -> Result<(), MigrationError> {
    db.execute(
        r#"
        CREATE TABLE orders (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            customer_id UUID NOT NULL,
            status VARCHAR(50) NOT NULL DEFAULT 'pending',
            total_amount DECIMAL(10,2) NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#,
        &[],
    )
    .await?;
    
    // Create indexes
    db.execute(
        "CREATE INDEX idx_orders_customer_id ON orders(customer_id)",
        &[],
    )
    .await?;
    
    db.execute(
        "CREATE INDEX idx_orders_status ON orders(status)",
        &[],
    )
    .await?;
    
    db.execute(
        "CREATE INDEX idx_orders_created_at ON orders(created_at)",
        &[],
    )
    .await?;
    
    Ok(())
}

#[migration(version = 2, description = "Add order_items table")]
#[depends_on(1)] // Depends on orders table migration
pub async fn create_order_items_table(db: &DatabaseConnection) -> Result<(), MigrationError> {
    db.execute(
        r#"
        CREATE TABLE order_items (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
            product_id UUID NOT NULL,
            quantity INTEGER NOT NULL CHECK (quantity > 0),
            unit_price DECIMAL(10,2) NOT NULL,
            total_price DECIMAL(10,2) NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#,
        &[],
    )
    .await?;
    
    db.execute(
        "CREATE INDEX idx_order_items_order_id ON order_items(order_id)",
        &[],
    )
    .await?;
    
    Ok(())
}

#[migration(version = 3, description = "Add shipping address to orders")]
pub async fn add_shipping_address_to_orders(db: &DatabaseConnection) -> Result<(), MigrationError> {
    // Add columns
    db.execute(
        r#"
        ALTER TABLE orders 
        ADD COLUMN shipping_street VARCHAR(200),
        ADD COLUMN shipping_city VARCHAR(100),
        ADD COLUMN shipping_postal_code VARCHAR(20),
        ADD COLUMN shipping_country VARCHAR(2)
        "#,
        &[],
    )
    .await?;
    
    Ok(())
}

#[rollback_migration(version = 3)]
pub async fn remove_shipping_address_from_orders(db: &DatabaseConnection) -> Result<(), MigrationError> {
    db.execute(
        r#"
        ALTER TABLE orders 
        DROP COLUMN shipping_street,
        DROP COLUMN shipping_city,
        DROP COLUMN shipping_postal_code,
        DROP COLUMN shipping_country
        "#,
        &[],
    )
    .await?;
    
    Ok(())
}

#[data_migration(version = 4, description = "Migrate legacy order data")]
pub async fn migrate_legacy_order_data(db: &DatabaseConnection) -> Result<(), MigrationError> {
    // Data migration from legacy system
    db.execute(
        r#"
        INSERT INTO orders (id, customer_id, status, total_amount, created_at)
        SELECT 
            legacy_id::UUID,
            customer_uuid,
            CASE 
                WHEN legacy_status = 'COMPLETED' THEN 'delivered'
                WHEN legacy_status = 'CANCELLED' THEN 'cancelled'
                ELSE 'pending'
            END,
            amount,
            legacy_created_at
        FROM legacy_orders
        WHERE migrated = false
        "#,
        &[],
    )
    .await?;
    
    // Mark as migrated
    db.execute(
        "UPDATE legacy_orders SET migrated = true WHERE migrated = false",
        &[],
    )
    .await?;
    
    Ok(())
}
```

**Migration Features**:
- **Version Management**: Automatic migration versioning
- **Dependency Tracking**: Migration dependencies
- **Rollback Support**: Automatic rollback migrations
- **Data Migrations**: Support for data transformation migrations
- **Transaction Safety**: Each migration runs in its own transaction

**When to Use**: For maintaining database schema evolution across deployments.

---

### `#[migration_runner]`

**Purpose**: Manages and executes database migrations automatically.

**Usage**:
```rust
#[migration_runner]
#[migration_table("schema_migrations")]
pub struct DatabaseMigrationRunner {
    connection: Arc<PgPool>,
}

#[migration_runner_impl]
impl DatabaseMigrationRunner {
    #[run_migrations]
    pub async fn migrate_database(&self) -> Result<MigrationResult, MigrationError> {
        // Automatically discovers and runs pending migrations
        let pending_migrations = self.discover_pending_migrations().await?;
        
        let mut applied_migrations = Vec::new();
        
        for migration in pending_migrations {
            tracing::info!("Running migration {}: {}", migration.version, migration.description);
            
            let start_time = Instant::now();
            
            // Run migration in transaction
            let mut tx = self.connection.begin().await?;
            
            match migration.execute(&mut tx).await {
                Ok(_) => {
                    // Record migration in schema table
                    self.record_migration(&mut tx, &migration).await?;
                    tx.commit().await?;
                    
                    let duration = start_time.elapsed();
                    tracing::info!("Migration {} completed in {:?}", migration.version, duration);
                    
                    applied_migrations.push(AppliedMigration {
                        version: migration.version,
                        description: migration.description.clone(),
                        duration,
                    });
                }
                Err(e) => {
                    tx.rollback().await?;
                    return Err(MigrationError::ExecutionFailed {
                        version: migration.version,
                        error: e.to_string(),
                    });
                }
            }
        }
        
        Ok(MigrationResult {
            applied_migrations,
            total_time: applied_migrations.iter().map(|m| m.duration).sum(),
        })
    }
    
    #[rollback_migrations]
    pub async fn rollback_to_version(&self, target_version: u32) -> Result<RollbackResult, MigrationError> {
        let current_version = self.get_current_version().await?;
        
        if target_version >= current_version {
            return Ok(RollbackResult::no_rollback_needed());
        }
        
        let migrations_to_rollback = self
            .get_applied_migrations_after_version(target_version)
            .await?;
        
        let mut rolled_back = Vec::new();
        
        for migration in migrations_to_rollback.into_iter().rev() {
            tracing::info!("Rolling back migration {}", migration.version);
            
            let mut tx = self.connection.begin().await?;
            
            match migration.rollback(&mut tx).await {
                Ok(_) => {
                    self.remove_migration_record(&mut tx, migration.version).await?;
                    tx.commit().await?;
                    
                    rolled_back.push(migration.version);
                }
                Err(e) => {
                    tx.rollback().await?;
                    return Err(MigrationError::RollbackFailed {
                        version: migration.version,
                        error: e.to_string(),
                    });
                }
            }
        }
        
        Ok(RollbackResult {
            rolled_back_migrations: rolled_back,
            final_version: target_version,
        })
    }
    
    #[migration_status]
    pub async fn get_migration_status(&self) -> Result<MigrationStatus, MigrationError> {
        let applied_migrations = self.get_applied_migrations().await?;
        let available_migrations = self.discover_all_migrations().await?;
        let pending_migrations = self.discover_pending_migrations().await?;
        
        Ok(MigrationStatus {
            current_version: applied_migrations.last().map(|m| m.version).unwrap_or(0),
            applied_count: applied_migrations.len(),
            pending_count: pending_migrations.len(),
            available_count: available_migrations.len(),
            applied_migrations,
            pending_migrations,
        })
    }
}
```

**When to Use**: For automated database schema management in CI/CD pipelines.

---

## Integration Examples

### Complete E-commerce Database Layer
```rust
#[connection_pool(
    database = "postgresql://user:pass@localhost/ecommerce",
    min_connections = 10,
    max_connections = 50,
    max_lifetime = "1h",
    idle_timeout = "10m"
)]
#[database_routing]
#[primary_db("postgresql://user:pass@primary:5432/ecommerce")]
#[replica_db("postgresql://user:pass@replica1:5432/ecommerce")]
#[replica_db("postgresql://user:pass@replica2:5432/ecommerce")]
pub struct EcommerceDatabase {
    primary: Arc<PgPool>,
    replicas: Vec<Arc<PgPool>>,
}

#[database_impl]
impl EcommerceDatabase {
    // Order Management
    #[route_to("primary")]
    #[transactional(isolation = "ReadCommitted")]
    pub async fn create_order(&self, order_data: CreateOrderRequest) -> Result<Order, DatabaseError> {
        // Create order record
        let order = self.insert_order(&order_data).await?;
        
        // Add order items
        for item in order_data.items {
            self.insert_order_item(&order.id, &item).await?;
        }
        
        // Update inventory
        self.update_inventory_levels(&order_data.items).await?;
        
        // Create audit log
        self.create_audit_entry("order_created", &order.id).await?;
        
        Ok(order)
    }
    
    #[prepared_statement(
        name = "insert_order",
        query = r#"
            INSERT INTO orders (customer_id, status, total_amount, shipping_address) 
            VALUES ($1, $2, $3, $4) 
            RETURNING id, created_at
        "#
    )]
    async fn insert_order(&self, order_data: &CreateOrderRequest) -> Result<Order, DatabaseError> {
        let row = self.primary
            .query_one_prepared(
                "insert_order",
                &[
                    &order_data.customer_id,
                    &"pending",
                    &order_data.total_amount,
                    &serde_json::to_value(&order_data.shipping_address)?,
                ]
            )
            .await?;
            
        Ok(Order {
            id: row.get("id"),
            customer_id: order_data.customer_id.clone(),
            status: OrderStatus::Pending,
            total_amount: order_data.total_amount,
            created_at: row.get("created_at"),
            shipping_address: order_data.shipping_address.clone(),
        })
    }
    
    #[route_to("replica")]
    #[read_only_transaction]
    #[query_cache(ttl = "5m", key = "order:{id}")]
    pub async fn find_order_by_id(&self, order_id: &str) -> Result<Option<Order>, DatabaseError> {
        let replica = self.get_replica();
        
        let row_opt = replica
            .query_opt("SELECT * FROM orders WHERE id = $1", &[&order_id])
            .await?;
            
        match row_opt {
            Some(row) => Ok(Some(Order::from_row(row)?)),
            None => Ok(None),
        }
    }
    
    #[route_to("replica")]
    #[read_only_transaction]
    #[query_cache(ttl = "2m", key = "customer_orders:{customer_id}:{limit}")]
    pub async fn find_orders_by_customer(
        &self,
        customer_id: &str,
        limit: i32
    ) -> Result<Vec<Order>, DatabaseError> {
        let replica = self.get_replica();
        
        let rows = replica
            .query(
                r#"
                SELECT * FROM orders 
                WHERE customer_id = $1 
                ORDER BY created_at DESC 
                LIMIT $2
                "#,
                &[&customer_id, &limit]
            )
            .await?;
            
        let orders = rows.into_iter()
            .map(Order::from_row)
            .collect::<Result<Vec<_>, _>>()?;
            
        Ok(orders)
    }
    
    // Product Management
    #[route_to("replica")]
    #[read_only_transaction]
    #[query_cache(ttl = "15m", key = "products:category:{category}")]
    pub async fn find_products_by_category(&self, category: &str) -> Result<Vec<Product>, DatabaseError> {
        let replica = self.get_replica();
        
        let rows = replica
            .query(
                r#"
                SELECT p.*, pi.quantity as inventory_quantity
                FROM products p
                LEFT JOIN product_inventory pi ON p.id = pi.product_id
                WHERE p.category = $1 AND p.active = true
                ORDER BY p.name
                "#,
                &[&category]
            )
            .await?;
            
        let products = rows.into_iter()
            .map(Product::from_row_with_inventory)
            .collect::<Result<Vec<_>, _>>()?;
            
        Ok(products)
    }
    
    #[route_to("primary")]
    #[transactional]
    pub async fn update_product_inventory(
        &self,
        product_id: &str,
        quantity_change: i32
    ) -> Result<i32, DatabaseError> {
        let new_quantity: i32 = self.primary
            .query_one(
                r#"
                UPDATE product_inventory 
                SET quantity = quantity + $1,
                    updated_at = NOW()
                WHERE product_id = $2
                RETURNING quantity
                "#,
                &[&quantity_change, &product_id]
            )
            .await?
            .get("quantity");
            
        // Invalidate related caches
        self.invalidate_product_cache(product_id).await?;
        
        Ok(new_quantity)
    }
    
    // Reporting Queries
    #[route_to("replica")]
    #[read_only_transaction]
    #[query_cache(ttl = "1h", key = "sales_report:{from}:{to}")]
    pub async fn generate_sales_report(
        &self,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>
    ) -> Result<SalesReport, DatabaseError> {
        let replica = self.get_replica();
        
        let sales_data = replica
            .query_one(
                r#"
                SELECT 
                    COUNT(*) as total_orders,
                    SUM(total_amount) as total_revenue,
                    AVG(total_amount) as average_order_value,
                    COUNT(DISTINCT customer_id) as unique_customers
                FROM orders
                WHERE created_at >= $1 AND created_at <= $2
                AND status != 'cancelled'
                "#,
                &[&from_date, &to_date]
            )
            .await?;
            
        Ok(SalesReport {
            period_start: from_date,
            period_end: to_date,
            total_orders: sales_data.get("total_orders"),
            total_revenue: sales_data.get("total_revenue"),
            average_order_value: sales_data.get("average_order_value"),
            unique_customers: sales_data.get("unique_customers"),
        })
    }
    
    // Health Check
    #[health_check]
    pub async fn check_database_health(&self) -> Result<DatabaseHealth, DatabaseError> {
        let primary_health = self.check_connection_health(&self.primary, "primary").await?;
        
        let mut replica_health = Vec::new();
        for (i, replica) in self.replicas.iter().enumerate() {
            let health = self.check_connection_health(replica, &format!("replica_{}", i)).await?;
            replica_health.push(health);
        }
        
        Ok(DatabaseHealth {
            primary: primary_health,
            replicas: replica_health,
        })
    }
    
    async fn check_connection_health(&self, pool: &PgPool, name: &str) -> Result<ConnectionHealth, DatabaseError> {
        let start_time = Instant::now();
        
        // Test connection with simple query
        pool.query_one("SELECT 1 as test", &[]).await?;
        
        let response_time = start_time.elapsed();
        let pool_status = pool.status();
        
        Ok(ConnectionHealth {
            name: name.to_string(),
            connected: true,
            response_time,
            pool_size: pool_status.size,
            idle_connections: pool_status.idle,
            active_connections: pool_status.size - pool_status.idle,
        })
    }
}

// Database Migrations for E-commerce
#[migration_runner]
pub struct EcommerceMigrationRunner {
    connection: Arc<PgPool>,
}

// Migration definitions would be included here
#[migration(version = 1, description = "Create initial e-commerce tables")]
pub async fn create_initial_tables(db: &DatabaseConnection) -> Result<(), MigrationError> {
    // Create all initial tables: orders, products, customers, etc.
    Ok(())
}
```

### Multi-Tenant Database Architecture
```rust
#[connection_pool(
    database = "postgresql://user:pass@localhost/multitenant",
    min_connections = 20,
    max_connections = 100
)]
#[database_routing]
pub struct MultiTenantDatabase {
    pool: Arc<PgPool>,
}

#[database_impl]
impl MultiTenantDatabase {
    #[transactional]
    #[tenant_aware] // Automatically adds tenant_id to queries
    pub async fn create_user(&self, tenant_id: &str, user_data: CreateUserRequest) -> Result<User, DatabaseError> {
        // tenant_id is automatically included in the query
        let user = self.pool
            .query_one(
                r#"
                INSERT INTO users (tenant_id, email, name, created_at)
                VALUES ($1, $2, $3, NOW())
                RETURNING id, created_at
                "#,
                &[&tenant_id, &user_data.email, &user_data.name]
            )
            .await?;
            
        Ok(User {
            id: user.get("id"),
            tenant_id: tenant_id.to_string(),
            email: user_data.email,
            name: user_data.name,
            created_at: user.get("created_at"),
        })
    }
    
    #[read_only_transaction]
    #[tenant_aware]
    #[query_cache(ttl = "10m", key = "tenant_users:{tenant_id}")]
    pub async fn find_users_by_tenant(&self, tenant_id: &str) -> Result<Vec<User>, DatabaseError> {
        let rows = self.pool
            .query(
                "SELECT * FROM users WHERE tenant_id = $1 ORDER BY created_at DESC",
                &[&tenant_id]
            )
            .await?;
            
        let users = rows.into_iter()
            .map(User::from_row)
            .collect::<Result<Vec<_>, _>>()?;
            
        Ok(users)
    }
    
    #[data_isolation] // Ensures data isolation between tenants
    pub async fn delete_tenant_data(&self, tenant_id: &str) -> Result<(), DatabaseError> {
        // Delete all data for a specific tenant
        let mut tx = self.pool.begin().await?;
        
        tx.execute("DELETE FROM user_sessions WHERE tenant_id = $1", &[&tenant_id]).await?;
        tx.execute("DELETE FROM user_preferences WHERE tenant_id = $1", &[&tenant_id]).await?;
        tx.execute("DELETE FROM users WHERE tenant_id = $1", &[&tenant_id]).await?;
        
        tx.commit().await?;
        
        // Invalidate tenant cache
        self.invalidate_tenant_cache(tenant_id).await?;
        
        Ok(())
    }
}
```

---

## Best Practices

### 1. Use Appropriate Transaction Boundaries
```rust
// Good: Fine-grained transactions
#[transactional] // Only for the specific operation
pub async fn update_user_profile() -> Result<(), Error> { }

// Avoid: Overly broad transactions  
// #[transactional] // Don't wrap entire service methods
// pub async fn complex_business_operation() -> Result<(), Error> { }
```

### 2. Choose Correct Isolation Levels
```rust
// For read-heavy operations
#[read_only_transaction]
pub async fn generate_report() -> Result<Report, Error> { }

// For operations requiring consistency
#[transactional(isolation = "RepeatableRead")]
pub async fn transfer_money() -> Result<(), Error> { }

// For high-concurrency operations
#[transactional(isolation = "ReadCommitted")] // Default
pub async fn create_order() -> Result<Order, Error> { }
```

### 3. Use Connection Routing Effectively
```rust
// Writes go to primary
#[route_to("primary")]
#[transactional]
pub async fn create_record() -> Result<(), Error> { }

// Reads can use replicas
#[route_to("replica")]
#[read_only_transaction]
pub async fn fetch_data() -> Result<Data, Error> { }

// Consistent reads need primary
#[route_to("primary")]
pub async fn read_after_write() -> Result<Data, Error> { }
```

### 4. Implement Proper Health Checks
```rust
#[health_check]
pub async fn check_database() -> DatabaseHealth {
    // Test actual connectivity
    // Check pool status
    // Verify query performance
    // Return meaningful health info
}
```

---

This completes the Database Macros documentation, providing comprehensive database integration capabilities for building robust, scalable data persistence layers in microservices.