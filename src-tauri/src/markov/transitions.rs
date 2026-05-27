use anyhow::Result;
use rusqlite::Connection;

use super::states::{encode_state, ALL_STATES};

pub struct TransitionMatrix {
    pub states: Vec<String>,
    pub matrix: Vec<Vec<f64>>,
    pub runs: Vec<Vec<f64>>,
}

pub fn compute_offense_transitions(
    conn: &Connection,
    season: i32,
    team_id: Option<i32>,
) -> Result<TransitionMatrix> {
    let n = ALL_STATES.len();
    let mut counts = vec![vec![0i64; n]; n];
    let mut runs_total = vec![vec![0.0f64; n]; n];

    let (query, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match team_id {
        Some(tid) => (
            "SELECT p.outs_before, p.bases_before, p.outs_after, p.bases_after, p.runs_scored
             FROM plays p
             JOIN games g ON g.game_pk = p.game_pk
             WHERE strftime('%Y', g.game_date) = ?1
               AND (g.home_team_id = ?2 OR g.away_team_id = ?2)",
            vec![
                Box::new(season.to_string()) as Box<dyn rusqlite::types::ToSql>,
                Box::new(tid as i64),
            ],
        ),
        None => (
            "SELECT p.outs_before, p.bases_before, p.outs_after, p.bases_after, p.runs_scored
             FROM plays p
             JOIN games g ON g.game_pk = p.game_pk
             WHERE strftime('%Y', g.game_date) = ?1",
            vec![Box::new(season.to_string()) as Box<dyn rusqlite::types::ToSql>],
        ),
    };

    let mut stmt = conn.prepare(query)?;

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut rows = stmt.query(param_refs.as_slice())?;

    while let Some(row) = rows.next()? {
        let outs_before: i32 = row.get(0)?;
        let bases_before: String = row.get(1)?;
        let outs_after: i32 = row.get(2)?;
        let bases_after: String = row.get(3)?;
        let runs_scored: i32 = row.get(4)?;

        let from_state = encode_state(outs_before, &bases_before);
        let to_state = encode_state(outs_after, &bases_after);

        let from_idx = match ALL_STATES.iter().position(|&s| s == from_state) {
            Some(i) => i,
            None => continue,
        };
        let to_idx = match ALL_STATES.iter().position(|&s| s == to_state) {
            Some(i) => i,
            None => continue,
        };

        counts[from_idx][to_idx] += 1;
        runs_total[from_idx][to_idx] += runs_scored as f64;
    }

    // Row-normalize to probabilities
    let mut matrix = vec![vec![0.0f64; n]; n];
    let mut runs = vec![vec![0.0f64; n]; n];

    for i in 0..n {
        let row_sum: i64 = counts[i].iter().sum();
        if row_sum > 0 {
            for j in 0..n {
                matrix[i][j] = counts[i][j] as f64 / row_sum as f64;
                if counts[i][j] > 0 {
                    runs[i][j] = runs_total[i][j] / counts[i][j] as f64;
                }
            }
        }
    }

    // Absorbing state: row is all zeros except self-loop = 1
    let abs_idx = n - 1;
    matrix[abs_idx][abs_idx] = 1.0;

    // Store in DB
    store_transitions(conn, season, team_id, &counts, &matrix, &runs)?;

    Ok(TransitionMatrix {
        states: ALL_STATES.iter().map(|s| s.to_string()).collect(),
        matrix,
        runs,
    })
}

fn store_transitions(
    conn: &Connection,
    season: i32,
    team_id: Option<i32>,
    counts: &[Vec<i64>],
    matrix: &[Vec<f64>],
    runs: &[Vec<f64>],
) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    // Clear existing
    match team_id {
        Some(tid) => {
            tx.execute(
                "DELETE FROM offense_transitions WHERE season = ?1 AND team_id = ?2",
                rusqlite::params![season, tid],
            )?;
        }
        None => {
            tx.execute(
                "DELETE FROM offense_transitions WHERE season = ?1 AND team_id IS NULL",
                [season],
            )?;
        }
    }

    let n = ALL_STATES.len();
    for i in 0..n {
        for j in 0..n {
            if counts[i][j] == 0 {
                continue;
            }
            tx.execute(
                "INSERT INTO offense_transitions
                 (season, team_id, state_from, state_to, occurrences, probability, avg_runs_scored)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    season,
                    team_id.map(|t| t as i64),
                    ALL_STATES[i],
                    ALL_STATES[j],
                    counts[i][j],
                    matrix[i][j],
                    runs[i][j],
                ],
            )?;
        }
    }

    tx.commit()?;
    Ok(())
}
