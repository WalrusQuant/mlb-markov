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
    pub frequency: f64,
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

    let row_totals: Vec<i64> = tm.counts.iter().map(|row| row.iter().sum()).collect();
    let grand_total: i64 = row_totals.iter().sum();

    let expected: Vec<StateExpectedRuns> = er
        .states
        .iter()
        .zip(er.labels.iter())
        .zip(er.values.iter())
        .enumerate()
        .map(|(i, ((s, l), v))| {
            let freq = if grand_total > 0 {
                row_totals[i] as f64 / grand_total as f64
            } else {
                0.0
            };
            StateExpectedRuns {
                state: s.clone(),
                label: l.clone(),
                expected_runs: (*v * 1000.0).round() / 1000.0,
                frequency: (freq * 10000.0).round() / 10000.0,
            }
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MomentumBundle {
    pub season: i32,
    pub team_id: Option<i32>,
    pub states: Vec<String>,
    pub cold_expected_runs: Vec<StateExpectedRuns>,
    pub hot_expected_runs: Vec<StateExpectedRuns>,
    pub cold_matrix: Vec<Vec<f64>>,
    pub hot_matrix: Vec<Vec<f64>>,
    pub cold_total_plays: i64,
    pub hot_total_plays: i64,
    pub verdict: String,
    pub overall_delta: f64,
}

fn build_expected_runs(
    tm: &transitions::TransitionMatrix,
) -> (Vec<StateExpectedRuns>, i64) {
    let er = expected_runs::compute_expected_runs(tm);
    let row_totals: Vec<i64> = tm.counts.iter().map(|row| row.iter().sum()).collect();
    let grand_total: i64 = row_totals.iter().sum();

    let runs: Vec<StateExpectedRuns> = er
        .states
        .iter()
        .zip(er.labels.iter())
        .zip(er.values.iter())
        .enumerate()
        .map(|(i, ((s, l), v))| {
            let freq = if grand_total > 0 {
                row_totals[i] as f64 / grand_total as f64
            } else {
                0.0
            };
            StateExpectedRuns {
                state: s.clone(),
                label: l.clone(),
                expected_runs: (*v * 1000.0).round() / 1000.0,
                frequency: (freq * 10000.0).round() / 10000.0,
            }
        })
        .collect();

    (runs, grand_total)
}

#[tauri::command]
pub fn get_momentum_analysis(
    state: State<'_, AppState>,
    season: Option<i32>,
    team_id: Option<i32>,
) -> Result<MomentumBundle, String> {
    let season = season.unwrap_or_else(crate::default_season);
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let (cold_tm, hot_tm) =
        transitions::compute_momentum_transitions(&conn, season, team_id)
            .map_err(|e| e.to_string())?;

    let (cold_er, cold_total) = build_expected_runs(&cold_tm);
    let (hot_er, hot_total) = build_expected_runs(&hot_tm);

    let cold_start = cold_er.iter().find(|r| r.state == "0_000")
        .map(|r| r.expected_runs).unwrap_or(0.0);
    let hot_start = hot_er.iter().find(|r| r.state == "0_000")
        .map(|r| r.expected_runs).unwrap_or(0.0);
    let overall_delta = ((hot_start - cold_start) * 1000.0).round() / 1000.0;

    let verdict = if overall_delta > 0.05 && cold_start.abs() > 1e-9 {
        format!("Momentum is real — teams score {:.1}% more when runs have already scored", (overall_delta / cold_start * 100.0))
    } else if overall_delta > 0.05 {
        "Momentum is real — teams score more when runs have already scored".to_string()
    } else if overall_delta < -0.05 {
        format!("Momentum is a myth — teams actually score less after scoring")
    } else {
        "No significant momentum effect — scoring doesn't change what happens next".to_string()
    };

    Ok(MomentumBundle {
        season,
        team_id,
        states: cold_tm.states,
        cold_expected_runs: cold_er,
        hot_expected_runs: hot_er,
        cold_matrix: cold_tm.matrix,
        hot_matrix: hot_tm.matrix,
        cold_total_plays: cold_total,
        hot_total_plays: hot_total,
        verdict,
        overall_delta,
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
