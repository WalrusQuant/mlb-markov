pub mod api;
pub mod commands;
pub mod db;
pub mod markov;

use std::sync::Mutex;

use tauri::Manager;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub db_path: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");

            let db_path = app_dir.join("mlbmarkov.db");
            let conn =
                db::init_db(&db_path).expect("failed to initialize database");

            let state = AppState {
                db: Mutex::new(conn),
                db_path: db_path.to_string_lossy().to_string(),
            };
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::data::get_db_status,
            commands::data::import_season,
            commands::offense::get_offense_transitions,
            commands::offense::get_teams,
            commands::pitching::search_pitchers,
            commands::pitching::get_pitch_sequences,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
