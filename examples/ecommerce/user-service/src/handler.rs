use anyhow::Result;
use chrono::Utc;
use mongodb::bson::{doc, oid::ObjectId};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    database::UserDatabase,
    model::{User, CreateUserRequest, UpdateUserRequest, UserResponse},
    utils::{hash_password, get_default_permissions},
};

pub struct UserHandler {
    db: UserDatabase,
}

impl UserHandler {
    pub fn new(db: UserDatabase) -> Self {
        Self { db }
    }

    pub async fn create_user(&self, request: CreateUserRequest) -> Result<UserResponse> {
        let hashed_password = hash_password(&request.password)?;
        let role = request.role.unwrap_or_else(|| "customer".to_string());
        let permissions = get_default_permissions(&role);

        let user = User {
            id: Uuid::new_v4().to_string(),
            email: request.email,
            name: request.name,
            role: role.clone(),
            permissions,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let collection = self.db.users();
        collection.insert_one(&user).await?;

        Ok(user.into())
    }

    pub async fn get_user(&self, user_id: &str) -> Result<Option<UserResponse>> {
        let collection = self.db.users();
        let filter = doc! { "id": user_id };
        
        if let Some(user) = collection.find_one(filter).await? {
            Ok(Some(user.into()))
        } else {
            Ok(None)
        }
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let collection = self.db.users();
        let filter = doc! { "email": email };
        
        let user = collection.find_one(filter).await?;
        Ok(user)
    }

    pub async fn update_user(&self, user_id: &str, request: UpdateUserRequest) -> Result<Option<UserResponse>> {
        let mut update_doc = doc! { "$set": { "updated_at": mongodb::bson::to_bson(&Utc::now())? } };

        if let Some(name) = &request.name {
            update_doc.get_document_mut("$set").unwrap().insert("name", name);
        }

        if let Some(role) = &request.role {
            update_doc.get_document_mut("$set").unwrap().insert("role", role);
            // Update permissions based on new role
            let permissions = get_default_permissions(role);
            update_doc.get_document_mut("$set").unwrap().insert("permissions", permissions);
        }

        if let Some(permissions) = &request.permissions {
            update_doc.get_document_mut("$set").unwrap().insert("permissions", permissions);
        }

        let collection = self.db.users();
        let filter = doc! { "id": user_id };
        
        collection.update_one(filter, update_doc).await?;
        
        // Return updated user
        self.get_user(user_id).await
    }

    pub async fn delete_user(&self, user_id: &str) -> Result<bool> {
        let collection = self.db.users();
        let filter = doc! { "id": user_id };
        
        let result = collection.delete_one(filter).await?;
        Ok(result.deleted_count > 0)
    }

    pub async fn list_users(&self, limit: i64, skip: u64) -> Result<Vec<UserResponse>> {
        let collection = self.db.users();
        let mut cursor = collection
            .find(doc! {})
            .await?;

        let mut users = Vec::new();
        while cursor.advance().await? {
            let user = cursor.deserialize_current()?;
            users.push(user.into());
        }

        Ok(users)
    }
}