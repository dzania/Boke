use crate::AppState;
use crate::error::ApiError;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use boke_core::models::Folder;
use serde::Deserialize;
// Folder handlers

pub async fn get_folders(State(state): State<AppState>) -> Result<Json<Vec<Folder>>, ApiError> {
    let folders = state.folder_service.get_folders().await?;
    Ok(Json(folders))
}

#[derive(Deserialize)]
pub struct CreateFolderRequest {
    name: String,
}

pub async fn create_folder(
    State(state): State<AppState>,
    Json(req): Json<CreateFolderRequest>,
) -> Result<Json<Folder>, ApiError> {
    let folder = state.folder_service.create_folder(&req.name).await?;
    Ok(Json(folder))
}

#[derive(Deserialize)]
pub struct RenameFolderRequest {
    name: String,
}

pub async fn rename_folder(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<RenameFolderRequest>,
) -> Result<StatusCode, ApiError> {
    state.folder_service.rename_folder(id, &req.name).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_folder(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    state.folder_service.delete_folder(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn move_feed_to_folder(
    State(state): State<AppState>,
    Path((folder_id, feed_id)): Path<(i64, i64)>,
) -> Result<StatusCode, ApiError> {
    state
        .folder_service
        .move_feed_to_folder(feed_id, Some(folder_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
