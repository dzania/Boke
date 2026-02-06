mod commands;
mod db;
mod feed;

use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
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
            commands::articles::mark_all_unread,
            commands::articles::toggle_favorite,
            commands::articles::get_favorites_count,
            commands::feeds::import_opml,
            commands::articles::search_articles,
            commands::articles::fetch_article_content,
            commands::folders::get_folders,
            commands::folders::create_folder,
            commands::folders::rename_folder,
            commands::folders::delete_folder,
            commands::folders::move_feed_to_folder,
        ])
        .setup(|app| {
            // Database
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");
            let db_path = app_dir.join("boke.db");

            let pool = tauri::async_runtime::block_on(db::init(&db_path))
                .expect("failed to initialize database");
            app.manage(pool);

            // System tray
            let refresh_item = MenuItemBuilder::with_id("refresh", "Refresh All").build(app)?;
            let open_item = MenuItemBuilder::with_id("open", "Open Boke").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

            let menu = MenuBuilder::new(app)
                .items(&[&refresh_item, &open_item, &quit_item])
                .build()?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Boke RSS Reader")
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "refresh" => {
                            // Emit event to frontend to trigger refresh
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray-refresh", ());
                            }
                        }
                        "open" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => app.exit(0),
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
