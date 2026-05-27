use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::api;
use crate::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DbStatus {
    pub db_path: String,
    pub games_count: i64,
    pub plays_count: i64,
    pub pitches_count: i64,
    pub teams_count: i64,
    pub players_count: i64,
    pub last_game_date: Option<String>,
    pub pending_games: i64,
}

#[tauri::command]
pub fn get_db_status(state: State<'_, AppState>) -> Result<DbStatus, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let count = |table: &str| -> i64 {
        conn.query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |row| {
            row.get(0)
        })
        .unwrap_or(0)
    };

    let last_game_date: Option<String> = conn
        .query_row(
            "SELECT MAX(game_date) FROM games WHERE data_fetched = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(None);

    let pending_games: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM games WHERE data_fetched = 0 AND status = 'Final'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(DbStatus {
        db_path: state.db_path.clone(),
        games_count: conn
            .query_row(
                "SELECT COUNT(*) FROM games WHERE data_fetched = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0),
        plays_count: count("plays"),
        pitches_count: count("pitches"),
        teams_count: count("teams"),
        players_count: count("players"),
        last_game_date,
        pending_games,
    })
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportProgress {
    current: i32,
    total: i32,
    game_pk: i64,
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub games_imported: i32,
    pub plays_inserted: i64,
    pub pitches_inserted: i64,
    pub games_skipped: i32,
}

#[tauri::command]
pub async fn import_season(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    season: Option<i32>,
) -> Result<ImportResult, String> {
    let season = season.unwrap_or_else(crate::default_season);
    let client = api::http_client().map_err(|e| e.to_string())?;

    // Step 1: Fetch schedule
    app_handle
        .emit(
            "import-progress",
            ImportProgress {
                current: 0,
                total: 0,
                game_pk: 0,
                message: format!("Fetching {} schedule...", season),
            },
        )
        .ok();

    let schedule = api::fetch_schedule(&client, season)
        .await
        .map_err(|e| format!("Failed to fetch schedule: {}", e))?;

    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        api::upsert_schedule(&conn, &schedule)
            .map_err(|e| format!("Failed to save schedule: {}", e))?;
    }

    // Step 2: Get games that need fetching
    let games_to_fetch: Vec<(i64, String)> = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT game_pk, game_date FROM games
                 WHERE data_fetched = 0 AND status = 'Final'
                 ORDER BY game_date",
            )
            .map_err(|e| e.to_string())?;
        let rows: Vec<(i64, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        rows
    };

    let total = games_to_fetch.len() as i32;
    let mut games_imported = 0i32;
    let mut games_skipped = 0i32;
    let mut total_plays = 0i64;
    let mut total_pitches = 0i64;

    // Step 3: Fetch play-by-play for each game
    for (i, (game_pk, game_date)) in games_to_fetch.iter().enumerate() {
        app_handle
            .emit(
                "import-progress",
                ImportProgress {
                    current: (i + 1) as i32,
                    total,
                    game_pk: *game_pk,
                    message: format!(
                        "Game {}/{}: {} ({})",
                        i + 1,
                        total,
                        game_pk,
                        game_date
                    ),
                },
            )
            .ok();

        // Rate limit: 1 second between requests
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;

        let raw = match api::fetch_play_by_play(&client, *game_pk).await {
            Ok(r) => r,
            Err(e) => {
                let msg = format!("Skipped game {} ({}): {}", game_pk, game_date, e);
                eprintln!("[mlb-markov] {}", msg);
                app_handle.emit("import-progress", ImportProgress {
                    current: (i + 1) as i32, total, game_pk: *game_pk,
                    message: msg,
                }).ok();
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                match api::fetch_play_by_play(&client, *game_pk).await {
                    Ok(r) => r,
                    Err(e2) => {
                        eprintln!("[mlb-markov] Retry failed for game {}: {}", game_pk, e2);
                        games_skipped += 1;
                        continue;
                    }
                }
            }
        };

        let parsed = api::parse_game(*game_pk, raw);

        let insert_result = {
            let conn = state.db.lock().map_err(|e| e.to_string())?;
            match api::insert_parsed_game(&conn, *game_pk, &parsed) {
                Ok(r) => Ok(r),
                Err(e) => {
                    let msg = format!("Skipped game {} ({}): insert error: {}", game_pk, game_date, e);
                    eprintln!("[mlb-markov] {}", msg);
                    app_handle.emit("import-progress", ImportProgress {
                        current: (i + 1) as i32, total, game_pk: *game_pk,
                        message: msg.clone(),
                    }).ok();
                    Err(msg)
                }
            }
        };

        let (plays, pitches) = match insert_result {
            Ok(r) => r,
            Err(_) => {
                games_skipped += 1;
                continue;
            }
        };

        total_plays += plays;
        total_pitches += pitches;
        games_imported += 1;
    }

    app_handle
        .emit(
            "import-progress",
            ImportProgress {
                current: total,
                total,
                game_pk: 0,
                message: format!(
                    "Done! {} games, {} plays, {} pitches",
                    games_imported, total_plays, total_pitches
                ),
            },
        )
        .ok();

    Ok(ImportResult {
        games_imported,
        plays_inserted: total_plays,
        pitches_inserted: total_pitches,
        games_skipped,
    })
}
