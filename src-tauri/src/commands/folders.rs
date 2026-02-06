//! Folder-related Tauri commands.

use boke_core::{DatabasePool, Folder, FolderService};
use tauri::State;

#[tauri::command]
pub async fn get_folders(
    svc: State<'_, FolderService<DatabasePool>>,
) -> Result<Vec<Folder>, String> {
    svc.get_folders().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_folder(
    name: String,
    svc: State<'_, FolderService<DatabasePool>>,
) -> Result<Folder, String> {
    svc.create_folder(&name).await.map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            format!("Folder \"{name}\" already exists")
        } else {
            e.to_string()
        }
    })
}

#[tauri::command]
pub async fn rename_folder(
    folder_id: i64,
    name: String,
    svc: State<'_, FolderService<DatabasePool>>,
) -> Result<(), String> {
    svc.rename_folder(folder_id, &name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_folder(
    folder_id: i64,
    svc: State<'_, FolderService<DatabasePool>>,
) -> Result<(), String> {
    svc.delete_folder(folder_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn move_feed_to_folder(
    feed_id: i64,
    folder_id: Option<i64>,
    svc: State<'_, FolderService<DatabasePool>>,
) -> Result<(), String> {
    svc.move_feed_to_folder(feed_id, folder_id)
        .await
        .map_err(|e| e.to_string())
}
