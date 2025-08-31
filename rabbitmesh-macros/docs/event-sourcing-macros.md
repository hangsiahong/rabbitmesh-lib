# Event Sourcing & CQRS Macros

Complete event sourcing implementation with command/query separation, aggregate management, event stores, read model projections, and domain event processing.

## Overview

Event sourcing macros provide a comprehensive toolkit for implementing event-driven architectures with CQRS patterns, enabling audit trails, temporal queries, and eventual consistency across distributed systems.

---

## Event Store Macros

### `#[event_store]`

**Purpose**: Creates an event store for persisting and retrieving domain events with optimized querying.

**Usage**:
```rust
#[event_store("order_events")]
#[storage(backend = "postgres", table = "order_event_stream")]
pub struct OrderEventStore {
    connection_pool: Arc<PgPool>,
    serializer: Arc<dyn EventSerializer>,
}

#[event_store_impl]
impl OrderEventStore {
    #[append_events]
    pub async fn append_order_events(
        &self,
        aggregate_id: &str,
        expected_version: u64,
        events: Vec<DomainEvent>
    ) -> Result<u64, EventStoreError> {
        // Automatic optimistic concurrency control
        // Event serialization and storage
        // Version management
        Ok(expected_version + events.len() as u64)
    }
    
    #[read_events]
    pub async fn read_order_events(
        &self,
        aggregate_id: &str,
        from_version: u64
    ) -> Result<Vec<EventEnvelope>, EventStoreError> {
        // Efficient event retrieval
        // Automatic deserialization
        // Streaming support for large event sequences
        todo!()
    }
    
    #[read_events_stream]
    pub async fn stream_order_events(
        &self,
        aggregate_id: &str,
        from_version: u64
    ) -> Result<impl Stream<Item = EventEnvelope>, EventStoreError> {
        // Streaming interface for memory efficiency
        todo!()
    }
    
    #[query_events]
    pub async fn query_events_by_type(
        &self,
        event_type: &str,
        from_timestamp: DateTime<Utc>,
        to_timestamp: DateTime<Utc>
    ) -> Result<Vec<EventEnvelope>, EventStoreError> {
        // Type-based event queries
        // Temporal range queries
        // Efficient indexing
        todo!()
    }
}
```

**Event Store Features**:
- **Optimistic Concurrency**: Version-based conflict detection
- **Event Serialization**: Pluggable serialization formats
- **Efficient Storage**: Optimized database schemas
- **Streaming Support**: Memory-efficient event streaming
- **Snapshot Support**: Aggregate snapshot storage
- **Indexing**: Optimized indexes for common query patterns

**When to Use**: As the primary storage mechanism for event-sourced systems.

---

### `#[event_serializer(format)]`

**Purpose**: Configures event serialization format and schema evolution.

**Usage**:
```rust
#[event_serializer("avro")]
#[schema_registry("http://schema-registry:8081")]
pub struct AvroEventSerializer {
    schema_cache: Arc<RwLock<HashMap<String, Schema>>>,
}

#[event_serializer("protobuf")]
#[schema_evolution(backward_compatible = true)]
pub struct ProtobufEventSerializer {
    descriptor_pool: Arc<DescriptorPool>,
}

#[event_serializer("json")]
#[versioning(strategy = "event_type_versioning")]
pub struct JsonEventSerializer {
    version_mapper: Arc<EventVersionMapper>,
}
```

**Serialization Options**:
- **JSON**: Human-readable, flexible schema evolution
- **Avro**: Compact, schema registry integration
- **Protocol Buffers**: Efficient, strong typing
- **MessagePack**: Compact binary format

**When to Use**: When you need specific serialization requirements or schema evolution support.

---

## Aggregate Macros

### `#[aggregate]`

**Purpose**: Implements aggregate root pattern with automatic event sourcing integration.

**Usage**:
```rust
#[aggregate]
#[derive(Debug, Clone)]
pub struct Order {
    id: OrderId,
    customer_id: CustomerId,
    items: Vec<OrderItem>,
    status: OrderStatus,
    total_amount: Money,
    created_at: DateTime<Utc>,
    version: u64,
}

#[aggregate_impl]
impl Order {
    #[command_handler]
    pub fn create_order(
        command: CreateOrderCommand
    ) -> Result<Vec<DomainEvent>, OrderError> {
        // Command validation
        if command.items.is_empty() {
            return Err(OrderError::EmptyOrder);
        }
        
        let total = command.items.iter()
            .map(|item| item.price * item.quantity)
            .sum();
        
        // Generate domain event
        Ok(vec![
            DomainEvent::OrderCreated(OrderCreated {
                order_id: command.order_id,
                customer_id: command.customer_id,
                items: command.items,
                total_amount: total,
                created_at: Utc::now(),
            })
        ])
    }
    
    #[command_handler]
    pub fn add_item(
        &self,
        command: AddItemCommand
    ) -> Result<Vec<DomainEvent>, OrderError> {
        // Business logic validation
        if self.status != OrderStatus::Draft {
            return Err(OrderError::OrderNotEditable);
        }
        
        if command.quantity <= 0 {
            return Err(OrderError::InvalidQuantity);
        }
        
        Ok(vec![
            DomainEvent::ItemAdded(ItemAdded {
                order_id: self.id,
                item: command.item,
                quantity: command.quantity,
                added_at: Utc::now(),
            })
        ])
    }
    
    #[command_handler]
    pub fn confirm_order(&self) -> Result<Vec<DomainEvent>, OrderError> {
        if self.status != OrderStatus::Draft {
            return Err(OrderError::OrderAlreadyConfirmed);
        }
        
        if self.items.is_empty() {
            return Err(OrderError::CannotConfirmEmptyOrder);
        }
        
        Ok(vec![
            DomainEvent::OrderConfirmed(OrderConfirmed {
                order_id: self.id,
                confirmed_at: Utc::now(),
                total_amount: self.total_amount,
            })
        ])
    }
    
    #[event_handler]
    fn apply_order_created(&mut self, event: OrderCreated) {
        self.id = event.order_id;
        self.customer_id = event.customer_id;
        self.items = event.items;
        self.total_amount = event.total_amount;
        self.status = OrderStatus::Draft;
        self.created_at = event.created_at;
    }
    
    #[event_handler]
    fn apply_item_added(&mut self, event: ItemAdded) {
        self.items.push(OrderItem {
            product_id: event.item.product_id,
            quantity: event.quantity,
            price: event.item.price,
        });
        
        self.total_amount = self.items.iter()
            .map(|item| item.price * item.quantity)
            .sum();
    }
    
    #[event_handler]
    fn apply_order_confirmed(&mut self, event: OrderConfirmed) {
        self.status = OrderStatus::Confirmed;
    }
}
```

**Aggregate Features**:
- **Command Handling**: Process commands and generate events
- **Event Application**: Apply events to rebuild aggregate state
- **Business Logic**: Encapsulate domain rules and invariants
- **Version Management**: Automatic version tracking
- **Conflict Resolution**: Handle concurrent modifications

**When to Use**: For implementing domain aggregates in event-sourced systems.

---

### `#[aggregate_repository]`

**Purpose**: Provides repository pattern for loading and saving aggregates with automatic event handling.

**Usage**:
```rust
#[aggregate_repository]
#[event_store("order_events")]
#[snapshot_store("order_snapshots", frequency = 100)]
pub struct OrderRepository {
    event_store: Arc<OrderEventStore>,
    snapshot_store: Arc<SnapshotStore>,
}

#[repository_impl]
impl OrderRepository {
    #[load_aggregate]
    pub async fn load_order(&self, order_id: OrderId) -> Result<Order, RepositoryError> {
        // 1. Try to load from snapshot
        if let Ok(snapshot) = self.snapshot_store.load(&order_id).await {
            let mut order = Order::from_snapshot(snapshot)?;
            
            // 2. Apply events since snapshot
            let events = self.event_store
                .read_events(&order_id.to_string(), order.version + 1)
                .await?;
                
            for event in events {
                order.apply_event(&event)?;
            }
            
            return Ok(order);
        }
        
        // 3. Load all events and rebuild aggregate
        let events = self.event_store
            .read_events(&order_id.to_string(), 0)
            .await?;
            
        if events.is_empty() {
            return Err(RepositoryError::AggregateNotFound);
        }
        
        let mut order = Order::default();
        for event in events {
            order.apply_event(&event)?;
        }
        
        Ok(order)
    }
    
    #[save_aggregate]
    pub async fn save_order(
        &self,
        order: &mut Order,
        expected_version: u64
    ) -> Result<(), RepositoryError> {
        let uncommitted_events = order.get_uncommitted_events();
        
        if uncommitted_events.is_empty() {
            return Ok(());
        }
        
        // Save events to event store
        let new_version = self.event_store
            .append_events(
                &order.id.to_string(),
                expected_version,
                uncommitted_events.clone()
            )
            .await?;
        
        // Update aggregate version
        order.mark_events_as_committed();
        order.set_version(new_version);
        
        // Create snapshot if needed
        if new_version % 100 == 0 {
            let snapshot = order.create_snapshot()?;
            self.snapshot_store.save(&order.id.to_string(), snapshot).await?;
        }
        
        Ok(())
    }
    
    #[exists]
    pub async fn order_exists(&self, order_id: &OrderId) -> Result<bool, RepositoryError> {
        let events = self.event_store
            .read_events(&order_id.to_string(), 0)
            .await?;
        Ok(!events.is_empty())
    }
}
```

**Repository Features**:
- **Aggregate Loading**: Load aggregates from events and snapshots
- **Optimistic Concurrency**: Version-based conflict detection
- **Snapshot Management**: Automatic snapshot creation and loading
- **Performance Optimization**: Efficient loading strategies
- **Event Handling**: Automatic uncommitted event management

**When to Use**: For managing aggregate persistence in event-sourced systems.

---

## CQRS Macros

### `#[command_handler]`

**Purpose**: Implements command handlers for processing commands and generating events.

**Usage**:
```rust
#[command_handler]
#[aggregate(Order)]
#[event_store("order_events")]
pub struct OrderCommandHandler {
    repository: Arc<OrderRepository>,
    event_publisher: Arc<EventPublisher>,
}

#[command_handler_impl]
impl OrderCommandHandler {
    #[handle(CreateOrderCommand)]
    #[validate]
    #[audit_log]
    pub async fn handle_create_order(
        &self,
        command: CreateOrderCommand
    ) -> Result<CommandResult, CommandError> {
        // Create new aggregate
        let events = Order::create_order(command)?;
        
        // Create and save new aggregate
        let mut order = Order::default();
        for event in &events {
            order.apply_event(event)?;
        }
        
        self.repository.save_order(&mut order, 0).await?;
        
        // Publish events
        for event in events {
            self.event_publisher.publish(event).await?;
        }
        
        Ok(CommandResult::success())
    }
    
    #[handle(AddItemCommand)]
    #[validate]
    #[require_auth]
    #[audit_log]
    pub async fn handle_add_item(
        &self,
        command: AddItemCommand
    ) -> Result<CommandResult, CommandError> {
        // Load existing aggregate
        let mut order = self.repository
            .load_order(command.order_id)
            .await?;
            
        let current_version = order.version;
        
        // Process command
        let events = order.add_item(command)?;
        
        // Apply events to aggregate
        for event in &events {
            order.apply_event(event)?;
        }
        
        // Save aggregate
        self.repository.save_order(&mut order, current_version).await?;
        
        // Publish events
        for event in events {
            self.event_publisher.publish(event).await?;
        }
        
        Ok(CommandResult::success())
    }
    
    #[handle(ConfirmOrderCommand)]
    #[validate]
    #[require_auth]
    #[require_permission("orders:confirm")]
    #[audit_log]
    pub async fn handle_confirm_order(
        &self,
        command: ConfirmOrderCommand
    ) -> Result<CommandResult, CommandError> {
        let mut order = self.repository
            .load_order(command.order_id)
            .await?;
            
        let current_version = order.version;
        let events = order.confirm_order()?;
        
        for event in &events {
            order.apply_event(event)?;
        }
        
        self.repository.save_order(&mut order, current_version).await?;
        
        for event in events {
            self.event_publisher.publish(event).await?;
        }
        
        Ok(CommandResult::success())
    }
}
```

**Command Handler Features**:
- **Command Processing**: Handle commands and coordinate aggregate operations
- **Event Publishing**: Automatically publish generated events
- **Validation**: Integrated command validation
- **Error Handling**: Robust error handling and recovery
- **Audit Logging**: Automatic audit trail generation

**When to Use**: For processing commands in CQRS architectures.

---

### `#[query_handler]`

**Purpose**: Implements query handlers for reading data from read models and projections.

**Usage**:
```rust
#[query_handler]
#[read_model("order_summaries")]
pub struct OrderQueryHandler {
    read_db: Arc<ReadDatabase>,
    cache: Arc<QueryCache>,
}

#[query_handler_impl]
impl OrderQueryHandler {
    #[handle(GetOrderQuery)]
    #[cached(ttl = "300s")]
    #[metrics]
    pub async fn handle_get_order(
        &self,
        query: GetOrderQuery
    ) -> Result<OrderSummary, QueryError> {
        let order = self.read_db
            .query_one(
                "SELECT * FROM order_summaries WHERE id = $1",
                &[&query.order_id]
            )
            .await?;
            
        Ok(OrderSummary::from_row(order)?)
    }
    
    #[handle(GetOrdersForCustomerQuery)]
    #[cached(ttl = "60s")]
    #[validate]
    #[require_auth]
    pub async fn handle_get_orders_for_customer(
        &self,
        query: GetOrdersForCustomerQuery
    ) -> Result<Vec<OrderSummary>, QueryError> {
        let orders = self.read_db
            .query(
                r#"
                SELECT * FROM order_summaries 
                WHERE customer_id = $1 
                ORDER BY created_at DESC 
                LIMIT $2 OFFSET $3
                "#,
                &[&query.customer_id, &query.limit, &query.offset]
            )
            .await?;
            
        let summaries = orders.into_iter()
            .map(OrderSummary::from_row)
            .collect::<Result<Vec<_>, _>>()?;
            
        Ok(summaries)
    }
    
    #[handle(GetOrderStatisticsQuery)]
    #[cached(ttl = "900s")]
    #[require_role("manager")]
    pub async fn handle_get_order_statistics(
        &self,
        query: GetOrderStatisticsQuery
    ) -> Result<OrderStatistics, QueryError> {
        let stats = self.read_db
            .query_one(
                r#"
                SELECT 
                    COUNT(*) as total_orders,
                    SUM(total_amount) as total_revenue,
                    AVG(total_amount) as average_order_value,
                    COUNT(DISTINCT customer_id) as unique_customers
                FROM order_summaries
                WHERE created_at >= $1 AND created_at <= $2
                "#,
                &[&query.from_date, &query.to_date]
            )
            .await?;
            
        Ok(OrderStatistics::from_row(stats)?)
    }
}
```

**Query Handler Features**:
- **Read Model Access**: Query optimized read models
- **Caching**: Automatic query result caching
- **Performance**: Optimized for read performance
- **Security**: Integrated authorization
- **Metrics**: Query performance monitoring

**When to Use**: For implementing the query side of CQRS architectures.

---

## Projection Macros

### `#[projection]`

**Purpose**: Creates read model projections that are automatically updated from domain events.

**Usage**:
```rust
#[projection("order_summaries")]
#[read_model(table = "order_summaries", database = "read_db")]
#[event_stream("order_events")]
pub struct OrderSummaryProjection {
    connection: Arc<PgPool>,
    metrics: Arc<ProjectionMetrics>,
}

#[projection_impl]
impl OrderSummaryProjection {
    #[event_handler(OrderCreated)]
    pub async fn handle_order_created(
        &self,
        event: OrderCreated
    ) -> Result<(), ProjectionError> {
        let summary = OrderSummary {
            id: event.order_id,
            customer_id: event.customer_id,
            total_amount: event.total_amount,
            item_count: event.items.len() as i32,
            status: "draft".to_string(),
            created_at: event.created_at,
            updated_at: event.created_at,
        };
        
        self.connection
            .execute(
                r#"
                INSERT INTO order_summaries 
                (id, customer_id, total_amount, item_count, status, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                &[
                    &summary.id,
                    &summary.customer_id,
                    &summary.total_amount,
                    &summary.item_count,
                    &summary.status,
                    &summary.created_at,
                    &summary.updated_at,
                ]
            )
            .await?;
            
        self.metrics.increment_projection_updates("order_created");
        Ok(())
    }
    
    #[event_handler(ItemAdded)]
    pub async fn handle_item_added(
        &self,
        event: ItemAdded
    ) -> Result<(), ProjectionError> {
        self.connection
            .execute(
                r#"
                UPDATE order_summaries 
                SET 
                    item_count = item_count + $2,
                    total_amount = total_amount + $3,
                    updated_at = $4
                WHERE id = $1
                "#,
                &[
                    &event.order_id,
                    &1,
                    &(event.item.price * event.quantity),
                    &event.added_at,
                ]
            )
            .await?;
            
        self.metrics.increment_projection_updates("item_added");
        Ok(())
    }
    
    #[event_handler(OrderConfirmed)]
    pub async fn handle_order_confirmed(
        &self,
        event: OrderConfirmed
    ) -> Result<(), ProjectionError> {
        self.connection
            .execute(
                r#"
                UPDATE order_summaries 
                SET 
                    status = 'confirmed',
                    updated_at = $2
                WHERE id = $1
                "#,
                &[&event.order_id, &event.confirmed_at]
            )
            .await?;
            
        self.metrics.increment_projection_updates("order_confirmed");
        Ok(())
    }
    
    #[rebuild_projection]
    pub async fn rebuild_from_events(&self) -> Result<(), ProjectionError> {
        // Clear existing projection data
        self.connection
            .execute("DELETE FROM order_summaries", &[])
            .await?;
        
        // Replay all events from the beginning
        let events = self.event_store
            .read_all_events_by_type("OrderCreated")
            .await?;
            
        for event in events {
            self.handle_order_created(event).await?;
        }
        
        // Continue with other event types...
        Ok(())
    }
}
```

**Projection Features**:
- **Automatic Updates**: Respond to domain events automatically
- **Read Optimization**: Optimized schemas for queries
- **Rebuild Capability**: Rebuild projections from event history
- **Error Handling**: Robust error handling and recovery
- **Monitoring**: Built-in projection health monitoring

**When to Use**: For creating optimized read models from event streams.

---

### `#[projection_manager]`

**Purpose**: Manages multiple projections and their synchronization state.

**Usage**:
```rust
#[projection_manager]
pub struct OrderProjectionManager {
    projections: Vec<Arc<dyn Projection>>,
    checkpoint_store: Arc<CheckpointStore>,
    event_stream: Arc<EventStream>,
}

#[projection_manager_impl]
impl OrderProjectionManager {
    #[register_projection(OrderSummaryProjection)]
    #[register_projection(CustomerOrderHistoryProjection)]
    #[register_projection(OrderAnalyticsProjection)]
    pub async fn start_projections(&self) -> Result<(), ProjectionError> {
        for projection in &self.projections {
            let checkpoint = self.checkpoint_store
                .get_checkpoint(projection.name())
                .await?
                .unwrap_or(0);
                
            self.start_projection_processing(projection.clone(), checkpoint).await?;
        }
        
        Ok(())
    }
    
    #[sync_projections]
    pub async fn sync_all_projections(&self) -> Result<(), ProjectionError> {
        let latest_event_position = self.event_stream.get_latest_position().await?;
        
        for projection in &self.projections {
            let checkpoint = self.checkpoint_store
                .get_checkpoint(projection.name())
                .await?
                .unwrap_or(0);
                
            if checkpoint < latest_event_position {
                self.catch_up_projection(projection.clone(), checkpoint).await?;
            }
        }
        
        Ok(())
    }
    
    #[health_check_projections]
    pub async fn check_projection_health(&self) -> Result<ProjectionHealth, ProjectionError> {
        let mut health = ProjectionHealth::new();
        
        for projection in &self.projections {
            let checkpoint = self.checkpoint_store
                .get_checkpoint(projection.name())
                .await?;
                
            let lag = self.calculate_projection_lag(projection.name(), checkpoint).await?;
            
            health.add_projection_status(
                projection.name(),
                checkpoint,
                lag,
                lag < Duration::from_secs(60) // Healthy if less than 1 minute behind
            );
        }
        
        Ok(health)
    }
}
```

**When to Use**: For coordinating multiple projections and monitoring their health.

---

## Event Publishing Macros

### `#[event_publisher]`

**Purpose**: Publishes domain events to event buses and external systems.

**Usage**:
```rust
#[event_publisher]
#[event_bus("rabbitmq")]
#[routing_key_strategy("event_type")]
pub struct DomainEventPublisher {
    message_bus: Arc<MessageBus>,
    serializer: Arc<EventSerializer>,
    metrics: Arc<PublisherMetrics>,
}

#[publisher_impl]
impl DomainEventPublisher {
    #[publish_event(OrderCreated)]
    #[routing_key("orders.created")]
    #[retry(attempts = 3)]
    pub async fn publish_order_created(
        &self,
        event: OrderCreated
    ) -> Result<(), PublishError> {
        let message = self.serialize_event(&event)?;
        
        self.message_bus
            .publish("orders.created", message)
            .await?;
            
        self.metrics.increment_published_events("OrderCreated");
        Ok(())
    }
    
    #[publish_event(OrderConfirmed)]
    #[routing_key("orders.confirmed")]
    #[delay("5s")] // Delayed publishing for eventual consistency
    pub async fn publish_order_confirmed(
        &self,
        event: OrderConfirmed
    ) -> Result<(), PublishError> {
        let message = self.serialize_event(&event)?;
        
        self.message_bus
            .publish_delayed("orders.confirmed", message, Duration::from_secs(5))
            .await?;
            
        Ok(())
    }
    
    #[publish_all_events]
    pub async fn publish_events(&self, events: Vec<DomainEvent>) -> Result<(), PublishError> {
        for event in events {
            match event {
                DomainEvent::OrderCreated(e) => self.publish_order_created(e).await?,
                DomainEvent::OrderConfirmed(e) => self.publish_order_confirmed(e).await?,
                DomainEvent::ItemAdded(e) => self.publish_item_added(e).await?,
                // ... handle other event types
            }
        }
        
        Ok(())
    }
}
```

**Event Publishing Features**:
- **Automatic Routing**: Route events based on type or content
- **Retry Logic**: Automatic retry on publication failures
- **Delayed Publishing**: Support for delayed event delivery
- **Serialization**: Pluggable event serialization
- **Monitoring**: Publication success/failure tracking

**When to Use**: For publishing domain events to external systems and services.

---

## Integration Examples

### Complete Event-Sourced Order Service
```rust
// Domain Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderEvent {
    OrderCreated(OrderCreated),
    ItemAdded(ItemAdded),
    ItemRemoved(ItemRemoved),
    OrderConfirmed(OrderConfirmed),
    OrderShipped(OrderShipped),
    OrderDelivered(OrderDelivered),
    OrderCancelled(OrderCancelled),
}

// Aggregate
#[aggregate]
#[derive(Debug, Clone)]
pub struct Order {
    id: OrderId,
    customer_id: CustomerId,
    items: Vec<OrderItem>,
    status: OrderStatus,
    total_amount: Money,
    created_at: DateTime<Utc>,
    version: u64,
    uncommitted_events: Vec<OrderEvent>,
}

// Command Handlers
#[service_impl]
impl OrderService {
    #[service_method("POST /orders")]
    #[require_auth]
    #[validate]
    #[command_handler]
    pub async fn create_order(msg: Message) -> Result<RpcResponse, String> {
        let command: CreateOrderCommand = msg.deserialize_payload()?;
        
        // Process command
        let command_handler = OrderCommandHandler::new(
            self.repository.clone(),
            self.event_publisher.clone()
        );
        
        let result = command_handler
            .handle_create_order(command)
            .await?;
        
        Ok(RpcResponse::success(&result, 0)?)
    }
    
    #[service_method("POST /orders/:id/items")]
    #[require_auth]
    #[require_ownership]
    #[validate]
    #[command_handler]
    pub async fn add_item_to_order(msg: Message) -> Result<RpcResponse, String> {
        let order_id = msg.get_path_param("id")?;
        let mut command: AddItemCommand = msg.deserialize_payload()?;
        command.order_id = order_id.parse()?;
        
        let command_handler = OrderCommandHandler::new(
            self.repository.clone(),
            self.event_publisher.clone()
        );
        
        let result = command_handler
            .handle_add_item(command)
            .await?;
        
        Ok(RpcResponse::success(&result, 0)?)
    }
    
    #[service_method("PUT /orders/:id/confirm")]
    #[require_auth]
    #[require_ownership]
    #[command_handler]
    pub async fn confirm_order(msg: Message) -> Result<RpcResponse, String> {
        let order_id = msg.get_path_param("id")?;
        let command = ConfirmOrderCommand {
            order_id: order_id.parse()?,
        };
        
        let command_handler = OrderCommandHandler::new(
            self.repository.clone(),
            self.event_publisher.clone()
        );
        
        let result = command_handler
            .handle_confirm_order(command)
            .await?;
        
        Ok(RpcResponse::success(&result, 0)?)
    }
    
    // Query Handlers
    #[service_method("GET /orders/:id")]
    #[require_auth]
    #[cached(ttl = "300s")]
    #[query_handler]
    pub async fn get_order(msg: Message) -> Result<RpcResponse, String> {
        let order_id = msg.get_path_param("id")?;
        let query = GetOrderQuery {
            order_id: order_id.parse()?,
        };
        
        let query_handler = OrderQueryHandler::new(self.read_db.clone());
        let order = query_handler.handle_get_order(query).await?;
        
        Ok(RpcResponse::success(&order, 0)?)
    }
    
    #[service_method("GET /customers/:id/orders")]
    #[require_auth]
    #[cached(ttl = "60s")]
    #[query_handler]
    pub async fn get_customer_orders(msg: Message) -> Result<RpcResponse, String> {
        let customer_id = msg.get_path_param("id")?;
        let limit = msg.get_query_param("limit")
            .unwrap_or("10".to_string())
            .parse()?;
        let offset = msg.get_query_param("offset")
            .unwrap_or("0".to_string())
            .parse()?;
        
        let query = GetOrdersForCustomerQuery {
            customer_id: customer_id.parse()?,
            limit,
            offset,
        };
        
        let query_handler = OrderQueryHandler::new(self.read_db.clone());
        let orders = query_handler.handle_get_orders_for_customer(query).await?;
        
        Ok(RpcResponse::success(&orders, 0)?)
    }
}

// Projections
#[projection("order_summaries")]
#[projection("customer_order_history")]
#[projection("order_analytics")]
pub struct OrderProjections {
    summary_projection: Arc<OrderSummaryProjection>,
    history_projection: Arc<CustomerOrderHistoryProjection>,
    analytics_projection: Arc<OrderAnalyticsProjection>,
}

// Event Handlers for External Integration
#[service_impl]
impl OrderEventHandlers {
    #[event_handler("OrderConfirmed")]
    pub async fn handle_order_confirmed(event: OrderConfirmed) -> Result<(), EventError> {
        // Trigger inventory reservation
        let inventory_command = ReserveInventoryCommand {
            order_id: event.order_id,
            items: fetch_order_items(&event.order_id).await?,
        };
        
        inventory_service
            .send_command("reserve_inventory", inventory_command)
            .await?;
        
        Ok(())
    }
    
    #[event_handler("OrderShipped")]
    pub async fn handle_order_shipped(event: OrderShipped) -> Result<(), EventError> {
        // Send shipping notification
        let notification = ShippingNotification {
            customer_id: event.customer_id,
            order_id: event.order_id,
            tracking_number: event.tracking_number,
            estimated_delivery: event.estimated_delivery,
        };
        
        notification_service
            .send_notification(notification)
            .await?;
        
        Ok(())
    }
}
```

### Multi-Service Event Choreography
```rust
// Order Service Events
#[event_publisher]
pub struct OrderEventPublisher {
    message_bus: Arc<MessageBus>,
}

#[publisher_impl]
impl OrderEventPublisher {
    #[publish_event(OrderConfirmed)]
    #[routing_key("orders.confirmed")]
    pub async fn publish_order_confirmed(&self, event: OrderConfirmed) -> Result<(), PublishError> {
        self.publish_to_bus("orders.confirmed", &event).await
    }
}

// Inventory Service Event Handlers
#[service_impl]
impl InventoryService {
    #[event_handler("orders.confirmed")]
    pub async fn handle_order_confirmed(event: OrderConfirmed) -> Result<(), EventError> {
        // Reserve inventory for confirmed order
        let reservation_result = reserve_inventory_for_order(&event).await?;
        
        if reservation_result.success {
            self.event_publisher
                .publish_inventory_reserved(InventoryReserved {
                    order_id: event.order_id,
                    reservation_id: reservation_result.reservation_id,
                })
                .await?;
        } else {
            self.event_publisher
                .publish_inventory_unavailable(InventoryUnavailable {
                    order_id: event.order_id,
                    unavailable_items: reservation_result.unavailable_items,
                })
                .await?;
        }
        
        Ok(())
    }
}

// Payment Service Event Handlers
#[service_impl]
impl PaymentService {
    #[event_handler("inventory.reserved")]
    pub async fn handle_inventory_reserved(event: InventoryReserved) -> Result<(), EventError> {
        // Charge payment for order with reserved inventory
        let charge_result = charge_payment_for_order(&event.order_id).await?;
        
        if charge_result.success {
            self.event_publisher
                .publish_payment_charged(PaymentCharged {
                    order_id: event.order_id,
                    payment_id: charge_result.payment_id,
                    amount: charge_result.amount,
                })
                .await?;
        } else {
            self.event_publisher
                .publish_payment_failed(PaymentFailed {
                    order_id: event.order_id,
                    error_code: charge_result.error_code,
                })
                .await?;
        }
        
        Ok(())
    }
    
    #[event_handler("inventory.unavailable")]
    pub async fn handle_inventory_unavailable(event: InventoryUnavailable) -> Result<(), EventError> {
        // Refund any pre-authorization for unavailable inventory
        refund_preauthorization(&event.order_id).await?;
        Ok(())
    }
}

// Shipping Service Event Handlers
#[service_impl]
impl ShippingService {
    #[event_handler("payment.charged")]
    pub async fn handle_payment_charged(event: PaymentCharged) -> Result<(), EventError> {
        // Create shipping label and schedule pickup
        let shipping_result = create_shipping_for_order(&event.order_id).await?;
        
        self.event_publisher
            .publish_shipping_scheduled(ShippingScheduled {
                order_id: event.order_id,
                tracking_number: shipping_result.tracking_number,
                estimated_delivery: shipping_result.estimated_delivery,
            })
            .await?;
        
        Ok(())
    }
}
```

---

## Best Practices

### 1. Design Events Carefully
```rust
// Good: Specific, immutable events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderConfirmed {
    pub order_id: OrderId,
    pub confirmed_at: DateTime<Utc>,
    pub total_amount: Money,
    pub confirmation_method: ConfirmationMethod,
}

// Avoid: Generic, mutable events
// pub struct OrderUpdated { pub changes: HashMap<String, Value> }
```

### 2. Implement Proper Error Handling
```rust
#[command_handler]
impl OrderCommandHandler {
    #[handle(CreateOrderCommand)]
    #[retry(attempts = 3)]
    #[timeout("30s")]
    pub async fn handle_create_order(&self, command: CreateOrderCommand) -> Result<CommandResult, CommandError> {
        // Proper error handling with retries and timeouts
    }
}
```

### 3. Use Snapshots for Large Aggregates
```rust
#[aggregate_repository]
#[snapshot_store("order_snapshots", frequency = 50)]
pub struct OrderRepository {
    // Automatic snapshot creation every 50 events
}
```

### 4. Monitor Projection Health
```rust
#[projection_manager]
impl OrderProjectionManager {
    #[health_check_projections]
    #[metrics]
    pub async fn check_health(&self) -> ProjectionHealth {
        // Monitor projection lag and health
    }
}
```

---

This completes the Event Sourcing & CQRS Macros documentation, providing comprehensive event-driven architecture capabilities for building scalable, auditable, and resilient microservices.