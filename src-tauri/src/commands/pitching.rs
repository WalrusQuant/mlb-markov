use std::collections::HashMap;

use serde::Serialize;
use tauri::State;

use crate::markov::{entropy, pitch_seq};
use crate::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PitcherSearchResult {
    pub player_id: i64,
    pub full_name: String,
    pub team: String,
    pub total_pitches: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PitchMatrixData {
    pub types: Vec<String>,
    pub matrix: Vec<Vec<f64>>,
    pub occurrences: Vec<Vec<i64>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CountEntropyData {
    pub count_state: String,
    pub entropy: f64,
    pub pitches: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PitchSequenceBundle {
    pub pitcher_id: i64,
    pub pitcher_name: String,
    pub season: i32,
    pub pitch_types: Vec<String>,
    pub overall_matrix: PitchMatrixData,
    pub by_count: HashMap<String, PitchMatrixData>,
    pub overall_entropy: f64,
    pub total_pitches: i64,
    pub count_entropy: Vec<CountEntropyData>,
}

#[tauri::command]
pub fn search_pitchers(
    state: State<'_, AppState>,
    query: String,
    season: i32,
) -> Result<Vec<PitcherSearchResult>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let search = format!("%{}%", query);
    let mut stmt = conn
        .prepare(
            "SELECT p.player_id, p.full_name, COALESCE(t.name, ''),
                    COUNT(DISTINCT pi.id) as pitch_count
             FROM players p
             JOIN plays pl ON pl.pitcher_id = p.player_id
             JOIN games g ON g.game_pk = pl.game_pk
             JOIN pitches pi ON pi.play_id = pl.id
             LEFT JOIN teams t ON t.team_id = p.team_id
             WHERE p.full_name LIKE ?1
               AND strftime('%Y', g.game_date) = ?2
             GROUP BY p.player_id
             HAVING pitch_count > 0
             ORDER BY pitch_count DESC
             LIMIT 20",
        )
        .map_err(|e| e.to_string())?;

    let results: Vec<PitcherSearchResult> = stmt
        .query_map(rusqlite::params![search, season.to_string()], |row| {
            Ok(PitcherSearchResult {
                player_id: row.get(0)?,
                full_name: row.get(1)?,
                team: row.get(2)?,
                total_pitches: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

#[tauri::command]
pub fn get_pitch_sequences(
    state: State<'_, AppState>,
    pitcher_id: i64,
    season: i32,
) -> Result<PitchSequenceBundle, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get pitcher name
    let pitcher_name: String = conn
        .query_row(
            "SELECT full_name FROM players WHERE player_id = ?1",
            [pitcher_id],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "Unknown".to_string());

    let transitions = pitch_seq::compute_pitch_transitions(&conn, pitcher_id, season)
        .map_err(|e| e.to_string())?;

    let profile = entropy::compute_pitcher_profile(&conn, pitcher_id, season, &transitions)
        .map_err(|e| e.to_string())?;

    let overall_matrix = PitchMatrixData {
        types: transitions.overall.types.clone(),
        matrix: transitions.overall.matrix.clone(),
        occurrences: transitions.overall.occurrences.clone(),
    };

    let by_count: HashMap<String, PitchMatrixData> = transitions
        .by_count
        .iter()
        .map(|(cs, mat)| {
            (
                cs.clone(),
                PitchMatrixData {
                    types: mat.types.clone(),
                    matrix: mat.matrix.clone(),
                    occurrences: mat.occurrences.clone(),
                },
            )
        })
        .collect();

    let count_entropy: Vec<CountEntropyData> = profile
        .by_count
        .iter()
        .map(|ce| CountEntropyData {
            count_state: ce.count_state.clone(),
            entropy: (ce.entropy * 1000.0).round() / 1000.0,
            pitches: ce.pitches,
        })
        .collect();

    Ok(PitchSequenceBundle {
        pitcher_id,
        pitcher_name,
        season,
        pitch_types: transitions.pitch_types,
        overall_matrix,
        by_count,
        overall_entropy: (profile.overall_entropy * 1000.0).round() / 1000.0,
        total_pitches: profile.total_pitches,
        count_entropy,
    })
}
