mod commands;
mod db;
mod feed;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            commands::feeds::add_feed,
            commands::feeds::remove_feed,
            commands::feeds::get_feeds,
            commands::feeds::refresh_feed,
            commands::feeds::refresh_all_feeds,
            commands::articles::get_articles,
            commands::articles::get_article,
            commands::articles::toggle_read,
            commands::articles::mark_all_read,
            commands::articles::toggle_favorite,
            commands::articles::search_articles,
        ])
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");
            let db_path = app_dir.join("boke.db");

            let pool = tauri::async_runtime::block_on(db::init(&db_path))
                .expect("failed to initialize database");
            app.manage(pool);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
