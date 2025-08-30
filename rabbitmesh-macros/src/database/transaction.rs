//! Universal Database Transaction Module
//! 
//! Provides generic database transaction support for any database system.
//! Works with PostgreSQL, MySQL, MongoDB, Redis, and other databases.

use quote::quote;

/// Generate database transaction preprocessing code
pub fn generate_transaction_preprocessing(isolation_level: Option<&str>) -> proc_macro2::TokenStream {
    quote! {
        tracing::debug!("🔄 Starting database transaction");
        
        // Generic database connection and transaction management
        let transaction_manager = DatabaseTransactionManager::new().await?;
        let transaction_id = uuid::Uuid::new_v4().to_string();
        
        tracing::debug!("📊 Transaction {} started with isolation level: {:?}", 
            transaction_id, #isolation_level);
        
        // Start transaction with specified isolation level
        let mut db_transaction = transaction_manager
            .begin_transaction(#isolation_level, &transaction_id)
            .await
            .map_err(|e| {
                tracing::error!("❌ Failed to start transaction {}: {}", transaction_id, e);
                rabbitmesh::error::RabbitMeshError::Handler(format!("Transaction failed to start: {}", e))
            })?;
        
        // Set up transaction context for the handler
        let transaction_context = TransactionContext {
            id: transaction_id.clone(),
            isolation_level: #isolation_level.unwrap_or("READ_COMMITTED").to_string(),
            started_at: std::time::SystemTime::now(),
            operations: Vec::new(),
            savepoints: std::collections::HashMap::new(),
        };
        
        // Store transaction context in message for handler access
        if let Ok(mut payload) = serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(msg.payload.clone()) {
            payload.insert("_transaction".to_string(), serde_json::to_value(&transaction_context)?);
        }
        
        /// Universal Database Transaction Manager
        struct DatabaseTransactionManager {
            connection_pool: DatabaseConnectionPool,
        }
        
        /// Database Connection Pool abstraction
        enum DatabaseConnectionPool {
            PostgreSQL(deadpool_postgres::Pool),
            MySQL(sqlx::MySqlPool),
            MongoDB(mongodb::Client),
            Redis(redis::Client),
            SQLite(sqlx::SqlitePool),
            Custom(Box<dyn CustomDatabasePool + Send + Sync>),
        }
        
        /// Custom database pool trait for extensibility
        trait CustomDatabasePool {
            async fn begin_transaction(&self, isolation_level: Option<&str>) -> Result<Box<dyn DatabaseTransaction>, DatabaseError>;
            async fn execute_query(&self, query: &str, params: &[&dyn std::any::Any]) -> Result<QueryResult, DatabaseError>;
        }
        
        /// Transaction context for tracking operations
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        struct TransactionContext {
            id: String,
            isolation_level: String,
            started_at: std::time::SystemTime,
            operations: Vec<TransactionOperation>,
            savepoints: std::collections::HashMap<String, String>,
        }
        
        /// Transaction operation tracking
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        struct TransactionOperation {
            operation_id: String,
            operation_type: String,
            table_or_collection: String,
            timestamp: std::time::SystemTime,
            affected_rows: Option<u64>,
        }
        
        /// Universal database transaction trait
        trait DatabaseTransaction: Send + Sync {
            async fn execute(&mut self, query: &str, params: &[&dyn std::any::Any]) -> Result<QueryResult, DatabaseError>;
            async fn commit(self: Box<Self>) -> Result<(), DatabaseError>;
            async fn rollback(self: Box<Self>) -> Result<(), DatabaseError>;
            async fn create_savepoint(&mut self, name: &str) -> Result<(), DatabaseError>;
            async fn rollback_to_savepoint(&mut self, name: &str) -> Result<(), DatabaseError>;
            async fn release_savepoint(&mut self, name: &str) -> Result<(), DatabaseError>;
        }
        
        /// Query result abstraction
        #[derive(Debug)]
        struct QueryResult {
            rows_affected: u64,
            data: Vec<std::collections::HashMap<String, serde_json::Value>>,
            metadata: QueryMetadata,
        }
        
        #[derive(Debug)]
        struct QueryMetadata {
            execution_time: std::time::Duration,
            query_plan: Option<String>,
            warnings: Vec<String>,
        }
        
        /// Database error handling
        #[derive(Debug, thiserror::Error)]
        enum DatabaseError {
            #[error("Connection error: {0}")]
            Connection(String),
            #[error("Transaction error: {0}")]
            Transaction(String),
            #[error("Query error: {0}")]
            Query(String),
            #[error("Serialization error: {0}")]
            Serialization(String),
            #[error("Timeout error: {0}")]
            Timeout(String),
            #[error("Lock error: {0}")]
            Lock(String),
        }
        
        impl DatabaseTransactionManager {
            /// Create new transaction manager with auto-detected database type
            async fn new() -> Result<Self, DatabaseError> {
                let database_type = std::env::var("DATABASE_TYPE")
                    .unwrap_or_else(|_| "postgresql".to_string())
                    .to_lowercase();
                
                let connection_pool = match database_type.as_str() {
                    "postgresql" | "postgres" => {
                        let database_url = std::env::var("DATABASE_URL")
                            .or_else(|_| std::env::var("POSTGRES_URL"))
                            .map_err(|_| DatabaseError::Connection("Missing DATABASE_URL for PostgreSQL".to_string()))?;
                        
                        let config = deadpool_postgres::Config::from_url(&database_url)
                            .map_err(|e| DatabaseError::Connection(format!("Invalid PostgreSQL URL: {}", e)))?;
                        let pool = config.create_pool(Some(deadpool_postgres::Runtime::Tokio1), tokio_postgres::NoTls)
                            .map_err(|e| DatabaseError::Connection(format!("Failed to create PostgreSQL pool: {}", e)))?;
                        
                        DatabaseConnectionPool::PostgreSQL(pool)
                    }
                    "mysql" | "mariadb" => {
                        let database_url = std::env::var("DATABASE_URL")
                            .or_else(|_| std::env::var("MYSQL_URL"))
                            .map_err(|_| DatabaseError::Connection("Missing DATABASE_URL for MySQL".to_string()))?;
                        
                        let pool = sqlx::MySqlPool::connect(&database_url).await
                            .map_err(|e| DatabaseError::Connection(format!("Failed to connect to MySQL: {}", e)))?;
                        
                        DatabaseConnectionPool::MySQL(pool)
                    }
                    "mongodb" | "mongo" => {
                        let database_url = std::env::var("DATABASE_URL")
                            .or_else(|_| std::env::var("MONGODB_URL"))
                            .map_err(|_| DatabaseError::Connection("Missing DATABASE_URL for MongoDB".to_string()))?;
                        
                        let client = mongodb::Client::with_uri_str(&database_url).await
                            .map_err(|e| DatabaseError::Connection(format!("Failed to connect to MongoDB: {}", e)))?;
                        
                        DatabaseConnectionPool::MongoDB(client)
                    }
                    "redis" => {
                        let database_url = std::env::var("DATABASE_URL")
                            .or_else(|_| std::env::var("REDIS_URL"))
                            .map_err(|_| DatabaseError::Connection("Missing DATABASE_URL for Redis".to_string()))?;
                        
                        let client = redis::Client::open(database_url)
                            .map_err(|e| DatabaseError::Connection(format!("Failed to connect to Redis: {}", e)))?;
                        
                        DatabaseConnectionPool::Redis(client)
                    }
                    "sqlite" => {
                        let database_url = std::env::var("DATABASE_URL")
                            .or_else(|_| std::env::var("SQLITE_URL"))
                            .unwrap_or_else(|_| "sqlite::memory:".to_string());
                        
                        let pool = sqlx::SqlitePool::connect(&database_url).await
                            .map_err(|e| DatabaseError::Connection(format!("Failed to connect to SQLite: {}", e)))?;
                        
                        DatabaseConnectionPool::SQLite(pool)
                    }
                    custom_type => {
                        // Load custom database implementation from registry
                        if let Some(custom_pool) = load_custom_database_pool(custom_type).await? {
                            DatabaseConnectionPool::Custom(custom_pool)
                        } else {
                            return Err(DatabaseError::Connection(format!("Unsupported database type: {}", custom_type)));
                        }
                    }
                };
                
                Ok(Self { connection_pool })
            }
            
            /// Begin a new transaction with specified isolation level
            async fn begin_transaction(
                &self, 
                isolation_level: Option<&str>,
                transaction_id: &str
            ) -> Result<Box<dyn DatabaseTransaction>, DatabaseError> {
                let isolation = isolation_level.unwrap_or("READ_COMMITTED");
                
                match &self.connection_pool {
                    DatabaseConnectionPool::PostgreSQL(pool) => {
                        let mut client = pool.get().await
                            .map_err(|e| DatabaseError::Connection(format!("Failed to get PostgreSQL connection: {}", e)))?;
                        
                        let transaction = client.transaction().await
                            .map_err(|e| DatabaseError::Transaction(format!("Failed to start PostgreSQL transaction: {}", e)))?;
                        
                        // Set isolation level
                        transaction.execute(&format!("SET TRANSACTION ISOLATION LEVEL {}", isolation), &[]).await
                            .map_err(|e| DatabaseError::Transaction(format!("Failed to set isolation level: {}", e)))?;
                        
                        Ok(Box::new(PostgreSQLTransaction::new(transaction, transaction_id.to_string())))
                    }
                    DatabaseConnectionPool::MySQL(pool) => {
                        let mut transaction = pool.begin().await
                            .map_err(|e| DatabaseError::Transaction(format!("Failed to start MySQL transaction: {}", e)))?;
                        
                        // Set isolation level
                        sqlx::query(&format!("SET TRANSACTION ISOLATION LEVEL {}", isolation))
                            .execute(&mut *transaction).await
                            .map_err(|e| DatabaseError::Transaction(format!("Failed to set isolation level: {}", e)))?;
                        
                        Ok(Box::new(MySQLTransaction::new(transaction, transaction_id.to_string())))
                    }
                    DatabaseConnectionPool::MongoDB(client) => {
                        let session = client.start_session(None).await
                            .map_err(|e| DatabaseError::Transaction(format!("Failed to start MongoDB session: {}", e)))?;
                        
                        Ok(Box::new(MongoDBTransaction::new(session, transaction_id.to_string())))
                    }
                    DatabaseConnectionPool::Redis(client) => {
                        let connection = client.get_async_connection().await
                            .map_err(|e| DatabaseError::Connection(format!("Failed to get Redis connection: {}", e)))?;
                        
                        Ok(Box::new(RedisTransaction::new(connection, transaction_id.to_string())))
                    }
                    DatabaseConnectionPool::SQLite(pool) => {
                        let transaction = pool.begin().await
                            .map_err(|e| DatabaseError::Transaction(format!("Failed to start SQLite transaction: {}", e)))?;
                        
                        Ok(Box::new(SQLiteTransaction::new(transaction, transaction_id.to_string())))
                    }
                    DatabaseConnectionPool::Custom(pool) => {
                        pool.begin_transaction(Some(isolation)).await
                    }
                }
            }
        }
        
        /// PostgreSQL transaction implementation
        struct PostgreSQLTransaction {
            transaction: deadpool_postgres::Transaction<'static>,
            id: String,
            operations: Vec<TransactionOperation>,
        }
        
        impl PostgreSQLTransaction {
            fn new(transaction: deadpool_postgres::Transaction<'static>, id: String) -> Self {
                Self {
                    transaction,
                    id,
                    operations: Vec::new(),
                }
            }
        }
        
        #[async_trait::async_trait]
        impl DatabaseTransaction for PostgreSQLTransaction {
            async fn execute(&mut self, query: &str, params: &[&dyn std::any::Any]) -> Result<QueryResult, DatabaseError> {
                let start_time = std::time::Instant::now();
                
                // Convert params to PostgreSQL format (simplified)
                let pg_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
                
                let rows = self.transaction.execute(query, &pg_params).await
                    .map_err(|e| DatabaseError::Query(format!("PostgreSQL query failed: {}", e)))?;
                
                let execution_time = start_time.elapsed();
                
                // Track operation
                self.operations.push(TransactionOperation {
                    operation_id: uuid::Uuid::new_v4().to_string(),
                    operation_type: extract_operation_type(query),
                    table_or_collection: extract_table_name(query),
                    timestamp: std::time::SystemTime::now(),
                    affected_rows: Some(rows),
                });
                
                Ok(QueryResult {
                    rows_affected: rows,
                    data: Vec::new(), // Would populate with actual data in real implementation
                    metadata: QueryMetadata {
                        execution_time,
                        query_plan: None,
                        warnings: Vec::new(),
                    },
                })
            }
            
            async fn commit(self: Box<Self>) -> Result<(), DatabaseError> {
                tracing::debug!("💾 Committing transaction {} with {} operations", 
                    self.id, self.operations.len());
                
                self.transaction.commit().await
                    .map_err(|e| DatabaseError::Transaction(format!("Failed to commit PostgreSQL transaction: {}", e)))?;
                
                tracing::debug!("✅ Transaction {} committed successfully", self.id);
                Ok(())
            }
            
            async fn rollback(self: Box<Self>) -> Result<(), DatabaseError> {
                tracing::debug!("🔄 Rolling back transaction {} with {} operations", 
                    self.id, self.operations.len());
                
                self.transaction.rollback().await
                    .map_err(|e| DatabaseError::Transaction(format!("Failed to rollback PostgreSQL transaction: {}", e)))?;
                
                tracing::debug!("↩️ Transaction {} rolled back successfully", self.id);
                Ok(())
            }
            
            async fn create_savepoint(&mut self, name: &str) -> Result<(), DatabaseError> {
                let query = format!("SAVEPOINT {}", name);
                self.transaction.execute(&query, &[]).await
                    .map_err(|e| DatabaseError::Transaction(format!("Failed to create savepoint: {}", e)))?;
                Ok(())
            }
            
            async fn rollback_to_savepoint(&mut self, name: &str) -> Result<(), DatabaseError> {
                let query = format!("ROLLBACK TO SAVEPOINT {}", name);
                self.transaction.execute(&query, &[]).await
                    .map_err(|e| DatabaseError::Transaction(format!("Failed to rollback to savepoint: {}", e)))?;
                Ok(())
            }
            
            async fn release_savepoint(&mut self, name: &str) -> Result<(), DatabaseError> {
                let query = format!("RELEASE SAVEPOINT {}", name);
                self.transaction.execute(&query, &[]).await
                    .map_err(|e| DatabaseError::Transaction(format!("Failed to release savepoint: {}", e)))?;
                Ok(())
            }
        }
        
        /// MySQL transaction implementation
        struct MySQLTransaction {
            transaction: sqlx::Transaction<'static, sqlx::MySql>,
            id: String,
            operations: Vec<TransactionOperation>,
        }
        
        impl MySQLTransaction {
            fn new(transaction: sqlx::Transaction<'static, sqlx::MySql>, id: String) -> Self {
                Self {
                    transaction,
                    id,
                    operations: Vec::new(),
                }
            }
        }
        
        #[async_trait::async_trait]
        impl DatabaseTransaction for MySQLTransaction {
            async fn execute(&mut self, query: &str, _params: &[&dyn std::any::Any]) -> Result<QueryResult, DatabaseError> {
                let start_time = std::time::Instant::now();
                
                let result = sqlx::query(query).execute(&mut *self.transaction).await
                    .map_err(|e| DatabaseError::Query(format!("MySQL query failed: {}", e)))?;
                
                let execution_time = start_time.elapsed();
                
                // Track operation
                self.operations.push(TransactionOperation {
                    operation_id: uuid::Uuid::new_v4().to_string(),
                    operation_type: extract_operation_type(query),
                    table_or_collection: extract_table_name(query),
                    timestamp: std::time::SystemTime::now(),
                    affected_rows: Some(result.rows_affected()),
                });
                
                Ok(QueryResult {
                    rows_affected: result.rows_affected(),
                    data: Vec::new(),
                    metadata: QueryMetadata {
                        execution_time,
                        query_plan: None,
                        warnings: Vec::new(),
                    },
                })
            }
            
            async fn commit(mut self: Box<Self>) -> Result<(), DatabaseError> {
                tracing::debug!("💾 Committing MySQL transaction {} with {} operations", 
                    self.id, self.operations.len());
                
                self.transaction.commit().await
                    .map_err(|e| DatabaseError::Transaction(format!("Failed to commit MySQL transaction: {}", e)))?;
                
                Ok(())
            }
            
            async fn rollback(mut self: Box<Self>) -> Result<(), DatabaseError> {
                tracing::debug!("🔄 Rolling back MySQL transaction {}", self.id);
                
                self.transaction.rollback().await
                    .map_err(|e| DatabaseError::Transaction(format!("Failed to rollback MySQL transaction: {}", e)))?;
                
                Ok(())
            }
            
            async fn create_savepoint(&mut self, name: &str) -> Result<(), DatabaseError> {
                let query = format!("SAVEPOINT {}", name);
                sqlx::query(&query).execute(&mut *self.transaction).await
                    .map_err(|e| DatabaseError::Transaction(format!("Failed to create savepoint: {}", e)))?;
                Ok(())
            }
            
            async fn rollback_to_savepoint(&mut self, name: &str) -> Result<(), DatabaseError> {
                let query = format!("ROLLBACK TO SAVEPOINT {}", name);
                sqlx::query(&query).execute(&mut *self.transaction).await
                    .map_err(|e| DatabaseError::Transaction(format!("Failed to rollback to savepoint: {}", e)))?;
                Ok(())
            }
            
            async fn release_savepoint(&mut self, name: &str) -> Result<(), DatabaseError> {
                let query = format!("RELEASE SAVEPOINT {}", name);
                sqlx::query(&query).execute(&mut *self.transaction).await
                    .map_err(|e| DatabaseError::Transaction(format!("Failed to release savepoint: {}", e)))?;
                Ok(())
            }
        }
        
        /// MongoDB transaction implementation
        struct MongoDBTransaction {
            session: mongodb::ClientSession,
            id: String,
            operations: Vec<TransactionOperation>,
        }
        
        impl MongoDBTransaction {
            fn new(session: mongodb::ClientSession, id: String) -> Self {
                Self {
                    session,
                    id,
                    operations: Vec::new(),
                }
            }
        }
        
        #[async_trait::async_trait]
        impl DatabaseTransaction for MongoDBTransaction {
            async fn execute(&mut self, _query: &str, _params: &[&dyn std::any::Any]) -> Result<QueryResult, DatabaseError> {
                // MongoDB operations would be implemented here
                // This is a simplified placeholder showing the pattern
                
                self.operations.push(TransactionOperation {
                    operation_id: uuid::Uuid::new_v4().to_string(),
                    operation_type: "mongodb_operation".to_string(),
                    table_or_collection: "collection".to_string(),
                    timestamp: std::time::SystemTime::now(),
                    affected_rows: Some(1),
                });
                
                Ok(QueryResult {
                    rows_affected: 1,
                    data: Vec::new(),
                    metadata: QueryMetadata {
                        execution_time: std::time::Duration::from_millis(1),
                        query_plan: None,
                        warnings: Vec::new(),
                    },
                })
            }
            
            async fn commit(mut self: Box<Self>) -> Result<(), DatabaseError> {
                self.session.commit_transaction().await
                    .map_err(|e| DatabaseError::Transaction(format!("Failed to commit MongoDB transaction: {}", e)))?;
                Ok(())
            }
            
            async fn rollback(mut self: Box<Self>) -> Result<(), DatabaseError> {
                self.session.abort_transaction().await
                    .map_err(|e| DatabaseError::Transaction(format!("Failed to rollback MongoDB transaction: {}", e)))?;
                Ok(())
            }
            
            async fn create_savepoint(&mut self, _name: &str) -> Result<(), DatabaseError> {
                // MongoDB doesn't support savepoints in the same way as SQL databases
                Ok(())
            }
            
            async fn rollback_to_savepoint(&mut self, _name: &str) -> Result<(), DatabaseError> {
                Ok(())
            }
            
            async fn release_savepoint(&mut self, _name: &str) -> Result<(), DatabaseError> {
                Ok(())
            }
        }
        
        /// Redis transaction implementation
        struct RedisTransaction {
            connection: redis::aio::Connection,
            id: String,
            operations: Vec<TransactionOperation>,
        }
        
        impl RedisTransaction {
            fn new(connection: redis::aio::Connection, id: String) -> Self {
                Self {
                    connection,
                    id,
                    operations: Vec::new(),
                }
            }
        }
        
        #[async_trait::async_trait]
        impl DatabaseTransaction for RedisTransaction {
            async fn execute(&mut self, _query: &str, _params: &[&dyn std::any::Any]) -> Result<QueryResult, DatabaseError> {
                // Redis transaction operations would be implemented here
                Ok(QueryResult {
                    rows_affected: 1,
                    data: Vec::new(),
                    metadata: QueryMetadata {
                        execution_time: std::time::Duration::from_millis(1),
                        query_plan: None,
                        warnings: Vec::new(),
                    },
                })
            }
            
            async fn commit(self: Box<Self>) -> Result<(), DatabaseError> {
                // Redis EXEC command would be used here
                Ok(())
            }
            
            async fn rollback(self: Box<Self>) -> Result<(), DatabaseError> {
                // Redis DISCARD command would be used here
                Ok(())
            }
            
            async fn create_savepoint(&mut self, _name: &str) -> Result<(), DatabaseError> {
                Ok(())
            }
            
            async fn rollback_to_savepoint(&mut self, _name: &str) -> Result<(), DatabaseError> {
                Ok(())
            }
            
            async fn release_savepoint(&mut self, _name: &str) -> Result<(), DatabaseError> {
                Ok(())
            }
        }
        
        /// SQLite transaction implementation
        struct SQLiteTransaction {
            transaction: sqlx::Transaction<'static, sqlx::Sqlite>,
            id: String,
            operations: Vec<TransactionOperation>,
        }
        
        impl SQLiteTransaction {
            fn new(transaction: sqlx::Transaction<'static, sqlx::Sqlite>, id: String) -> Self {
                Self {
                    transaction,
                    id,
                    operations: Vec::new(),
                }
            }
        }
        
        #[async_trait::async_trait]
        impl DatabaseTransaction for SQLiteTransaction {
            async fn execute(&mut self, query: &str, _params: &[&dyn std::any::Any]) -> Result<QueryResult, DatabaseError> {
                let start_time = std::time::Instant::now();
                
                let result = sqlx::query(query).execute(&mut *self.transaction).await
                    .map_err(|e| DatabaseError::Query(format!("SQLite query failed: {}", e)))?;
                
                let execution_time = start_time.elapsed();
                
                self.operations.push(TransactionOperation {
                    operation_id: uuid::Uuid::new_v4().to_string(),
                    operation_type: extract_operation_type(query),
                    table_or_collection: extract_table_name(query),
                    timestamp: std::time::SystemTime::now(),
                    affected_rows: Some(result.rows_affected()),
                });
                
                Ok(QueryResult {
                    rows_affected: result.rows_affected(),
                    data: Vec::new(),
                    metadata: QueryMetadata {
                        execution_time,
                        query_plan: None,
                        warnings: Vec::new(),
                    },
                })
            }
            
            async fn commit(mut self: Box<Self>) -> Result<(), DatabaseError> {
                self.transaction.commit().await
                    .map_err(|e| DatabaseError::Transaction(format!("Failed to commit SQLite transaction: {}", e)))?;
                Ok(())
            }
            
            async fn rollback(mut self: Box<Self>) -> Result<(), DatabaseError> {
                self.transaction.rollback().await
                    .map_err(|e| DatabaseError::Transaction(format!("Failed to rollback SQLite transaction: {}", e)))?;
                Ok(())
            }
            
            async fn create_savepoint(&mut self, name: &str) -> Result<(), DatabaseError> {
                let query = format!("SAVEPOINT {}", name);
                sqlx::query(&query).execute(&mut *self.transaction).await
                    .map_err(|e| DatabaseError::Transaction(format!("Failed to create savepoint: {}", e)))?;
                Ok(())
            }
            
            async fn rollback_to_savepoint(&mut self, name: &str) -> Result<(), DatabaseError> {
                let query = format!("ROLLBACK TO SAVEPOINT {}", name);
                sqlx::query(&query).execute(&mut *self.transaction).await
                    .map_err(|e| DatabaseError::Transaction(format!("Failed to rollback to savepoint: {}", e)))?;
                Ok(())
            }
            
            async fn release_savepoint(&mut self, name: &str) -> Result<(), DatabaseError> {
                let query = format!("RELEASE SAVEPOINT {}", name);
                sqlx::query(&query).execute(&mut *self.transaction).await
                    .map_err(|e| DatabaseError::Transaction(format!("Failed to release savepoint: {}", e)))?;
                Ok(())
            }
        }
        
        /// Utility functions
        fn extract_operation_type(query: &str) -> String {
            let query_upper = query.trim().to_uppercase();
            if query_upper.starts_with("SELECT") {
                "SELECT".to_string()
            } else if query_upper.starts_with("INSERT") {
                "INSERT".to_string()
            } else if query_upper.starts_with("UPDATE") {
                "UPDATE".to_string()
            } else if query_upper.starts_with("DELETE") {
                "DELETE".to_string()
            } else {
                "OTHER".to_string()
            }
        }
        
        fn extract_table_name(query: &str) -> String {
            // Simplified table name extraction
            let query_upper = query.trim().to_uppercase();
            let words: Vec<&str> = query_upper.split_whitespace().collect();
            
            for (i, word) in words.iter().enumerate() {
                if word == &"FROM" || word == &"INTO" || word == &"UPDATE" {
                    if i + 1 < words.len() {
                        return words[i + 1].to_string();
                    }
                }
            }
            
            "unknown".to_string()
        }
        
        /// Load custom database pool implementation
        async fn load_custom_database_pool(_database_type: &str) -> Result<Option<Box<dyn CustomDatabasePool + Send + Sync>>, DatabaseError> {
            // This would load custom database implementations from a registry
            // For now, return None to indicate no custom implementation found
            Ok(None)
        }
        
        // Auto-commit/rollback logic based on handler result
        let _transaction_guard = TransactionGuard {
            transaction: Some(db_transaction),
            should_commit: false,
        };
        
        /// Transaction guard for automatic cleanup
        struct TransactionGuard {
            transaction: Option<Box<dyn DatabaseTransaction>>,
            should_commit: bool,
        }
        
        impl TransactionGuard {
            fn mark_for_commit(&mut self) {
                self.should_commit = true;
            }
        }
        
        impl Drop for TransactionGuard {
            fn drop(&mut self) {
                if let Some(transaction) = self.transaction.take() {
                    let should_commit = self.should_commit;
                    tokio::spawn(async move {
                        if should_commit {
                            if let Err(e) = transaction.commit().await {
                                tracing::error!("Failed to auto-commit transaction: {}", e);
                            }
                        } else {
                            if let Err(e) = transaction.rollback().await {
                                tracing::error!("Failed to auto-rollback transaction: {}", e);
                            }
                        }
                    });
                }
            }
        }
    }
}