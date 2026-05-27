use serde::Serialize;
use tauri::State;

use crate::markov::{expected_runs, transitions};
use crate::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateExpectedRuns {
    pub state: String,
    pub label: String,
    pub expected_runs: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OffenseBundle {
    pub season: i32,
    pub team_id: Option<i32>,
    pub states: Vec<String>,
    pub matrix: Vec<Vec<f64>>,
    pub expected_runs: Vec<StateExpectedRuns>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamOption {
    pub team_id: i32,
    pub name: String,
    pub abbreviation: String,
}

#[tauri::command]
pub fn get_offense_transitions(
    state: State<'_, AppState>,
    season: Option<i32>,
    team_id: Option<i32>,
) -> Result<OffenseBundle, String> {
    let season = season.unwrap_or_else(crate::default_season);
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let tm = transitions::compute_offense_transitions(&conn, season, team_id)
        .map_err(|e| e.to_string())?;

    let er = expected_runs::compute_expected_runs(&tm);

    let expected: Vec<StateExpectedRuns> = er
        .states
        .iter()
        .zip(er.labels.iter())
        .zip(er.values.iter())
        .map(|((s, l), v)| StateExpectedRuns {
            state: s.clone(),
            label: l.clone(),
            expected_runs: (*v * 1000.0).round() / 1000.0,
        })
        .collect();

    Ok(OffenseBundle {
        season,
        team_id,
        states: tm.states,
        matrix: tm.matrix,
        expected_runs: expected,
    })
}

#[tauri::command]
pub fn get_teams(state: State<'_, AppState>) -> Result<Vec<TeamOption>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT team_id, name, abbreviation FROM teams ORDER BY name")
        .map_err(|e| e.to_string())?;

    let teams: Vec<TeamOption> = stmt
        .query_map([], |row| {
            Ok(TeamOption {
                team_id: row.get(0)?,
                name: row.get(1)?,
                abbreviation: row.get::<_, String>(2).unwrap_or_default(),
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(teams)
}
