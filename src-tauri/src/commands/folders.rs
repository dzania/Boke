use serde::Serialize;
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct Folder {
    pub id: i64,
    pub name: String,
    pub feed_count: i64,
}

#[tauri::command]
pub async fn get_folders(pool: State<'_, SqlitePool>) -> Result<Vec<Folder>, String> {
    let rows = sqlx::query_as::<_, (i64, String)>(
        "SELECT id, name FROM folders ORDER BY name COLLATE NOCASE",
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let mut folders = Vec::with_capacity(rows.len());
    for (id, name) in rows {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM feeds WHERE folder_id = ?")
            .bind(id)
            .fetch_one(pool.inner())
            .await
            .map_err(|e| e.to_string())?;

        folders.push(Folder {
            id,
            name,
            feed_count: count,
        });
    }

    Ok(folders)
}

#[tauri::command]
pub async fn create_folder(name: String, pool: State<'_, SqlitePool>) -> Result<Folder, String> {
    let id = sqlx::query_scalar::<_, i64>("INSERT INTO folders (name) VALUES (?) RETURNING id")
        .bind(&name)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                format!("Folder \"{name}\" already exists")
            } else {
                e.to_string()
            }
        })?;

    Ok(Folder {
        id,
        name,
        feed_count: 0,
    })
}

#[tauri::command]
pub async fn rename_folder(
    folder_id: i64,
    name: String,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    sqlx::query("UPDATE folders SET name = ? WHERE id = ?")
        .bind(&name)
        .bind(folder_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_folder(folder_id: i64, pool: State<'_, SqlitePool>) -> Result<(), String> {
    // Unassign feeds first (set folder_id to NULL), then delete folder
    sqlx::query("UPDATE feeds SET folder_id = NULL WHERE folder_id = ?")
        .bind(folder_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM folders WHERE id = ?")
        .bind(folder_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn move_feed_to_folder(
    feed_id: i64,
    folder_id: Option<i64>,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    sqlx::query("UPDATE feeds SET folder_id = ? WHERE id = ?")
        .bind(folder_id)
        .bind(feed_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
