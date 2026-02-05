use serde::Serialize;
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub feed_ids: Vec<i64>,
}

#[tauri::command]
pub async fn get_tags(pool: State<'_, SqlitePool>) -> Result<Vec<Tag>, String> {
    let rows = sqlx::query_as::<_, (i64, String)>("SELECT id, name FROM tags ORDER BY name COLLATE NOCASE")
        .fetch_all(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    let mut tags = Vec::with_capacity(rows.len());
    for (id, name) in rows {
        let feed_ids: Vec<i64> =
            sqlx::query_scalar("SELECT feed_id FROM feed_tags WHERE tag_id = ?")
                .bind(id)
                .fetch_all(pool.inner())
                .await
                .map_err(|e| e.to_string())?;

        tags.push(Tag { id, name, feed_ids });
    }

    Ok(tags)
}

#[tauri::command]
pub async fn create_tag(name: String, pool: State<'_, SqlitePool>) -> Result<Tag, String> {
    let id = sqlx::query_scalar::<_, i64>("INSERT INTO tags (name) VALUES (?) RETURNING id")
        .bind(&name)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                format!("Tag \"{name}\" already exists")
            } else {
                e.to_string()
            }
        })?;

    Ok(Tag {
        id,
        name,
        feed_ids: vec![],
    })
}

#[tauri::command]
pub async fn tag_feed(feed_id: i64, tag_id: i64, pool: State<'_, SqlitePool>) -> Result<(), String> {
    sqlx::query("INSERT OR IGNORE INTO feed_tags (feed_id, tag_id) VALUES (?, ?)")
        .bind(feed_id)
        .bind(tag_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn untag_feed(feed_id: i64, tag_id: i64, pool: State<'_, SqlitePool>) -> Result<(), String> {
    sqlx::query("DELETE FROM feed_tags WHERE feed_id = ? AND tag_id = ?")
        .bind(feed_id)
        .bind(tag_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_tag(tag_id: i64, pool: State<'_, SqlitePool>) -> Result<(), String> {
    sqlx::query("DELETE FROM tags WHERE id = ?")
        .bind(tag_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
