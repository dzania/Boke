use crate::db::{Database, DbResult};
use crate::models::Folder;
use std::sync::Arc;

pub struct FolderService<D: Database> {
    db: Arc<D>,
}

impl<D: Database> FolderService<D> {
    pub fn new(db: Arc<D>) -> Self {
        Self { db }
    }

    pub async fn get_folders(&self) -> DbResult<Vec<Folder>> {
        self.db.get_folders().await
    }

    pub async fn create_folder(&self, name: &str) -> DbResult<Folder> {
        self.db.create_folder(name).await
    }

    pub async fn rename_folder(&self, id: i64, name: &str) -> DbResult<()> {
        self.db.rename_folder(id, name).await
    }

    pub async fn delete_folder(&self, id: i64) -> DbResult<()> {
        self.db.delete_folder(id).await
    }

    pub async fn move_feed_to_folder(&self, feed_id: i64, folder_id: Option<i64>) -> DbResult<()> {
        self.db.move_feed_to_folder(feed_id, folder_id).await
    }
}
