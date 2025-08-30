use anyhow::Result;
use mongodb::{Client, Collection, Database};
use crate::{config::Settings, model::User};

pub struct UserDatabase {
    db: Database,
}

impl UserDatabase {
    pub async fn new(settings: &Settings) -> Result<Self> {
        let client = Client::with_uri_str(&settings.database.url).await?;
        let db = client.database(&settings.database.name);
        
        // Create indexes
        let users: Collection<User> = db.collection("users");
        
        Ok(Self { db })
    }

    pub fn users(&self) -> Collection<User> {
        self.db.collection("users")
    }
}