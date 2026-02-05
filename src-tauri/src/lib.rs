mod db;
mod feed;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
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
