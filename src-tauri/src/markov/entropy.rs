use anyhow::Result;
use rusqlite::Connection;

use super::pitch_seq::PitchTransitionSet;

pub struct PitcherProfile {
    pub pitcher_id: i64,
    pub season: i32,
    pub overall_entropy: f64,
    pub total_pitches: i64,
    pub by_count: Vec<CountEntropy>,
}

pub struct CountEntropy {
    pub count_state: String,
    pub entropy: f64,
    pub pitches: i64,
}

pub fn shannon_entropy(probs: &[f64]) -> f64 {
    -probs
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| p * p.ln())
        .sum::<f64>()
}

pub fn compute_pitcher_profile(
    conn: &Connection,
    pitcher_id: i64,
    season: i32,
    transitions: &PitchTransitionSet,
) -> Result<PitcherProfile> {
    // Overall entropy: flatten the matrix into a pitch distribution
    let overall_entropy = matrix_entropy(&transitions.overall.matrix);
    let total_pitches: i64 = transitions
        .overall
        .occurrences
        .iter()
        .flat_map(|row| row.iter())
        .sum();

    let mut by_count: Vec<CountEntropy> = Vec::new();
    for (cs, mat) in &transitions.by_count {
        let e = matrix_entropy(&mat.matrix);
        let pitches: i64 = mat.occurrences.iter().flat_map(|row| row.iter()).sum();
        by_count.push(CountEntropy {
            count_state: cs.clone(),
            entropy: e,
            pitches,
        });
    }
    by_count.sort_by(|a, b| a.count_state.cmp(&b.count_state));

    // Store in DB
    store_profile(conn, pitcher_id, season, overall_entropy, total_pitches, &by_count)?;

    Ok(PitcherProfile {
        pitcher_id,
        season,
        overall_entropy,
        total_pitches,
        by_count,
    })
}

/// Average row entropy weighted by row occurrence count
fn matrix_entropy(matrix: &[Vec<f64>]) -> f64 {
    let mut total_weight = 0.0;
    let mut weighted_entropy = 0.0;

    for row in matrix {
        let row_sum: f64 = row.iter().sum();
        if row_sum <= 0.0 {
            continue;
        }
        let e = shannon_entropy(row);
        weighted_entropy += row_sum * e;
        total_weight += row_sum;
    }

    if total_weight > 0.0 {
        weighted_entropy / total_weight
    } else {
        0.0
    }
}

fn store_profile(
    conn: &Connection,
    pitcher_id: i64,
    season: i32,
    overall_entropy: f64,
    total_pitches: i64,
    by_count: &[CountEntropy],
) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    tx.execute(
        "DELETE FROM pitcher_profiles WHERE pitcher_id = ?1 AND season = ?2",
        rusqlite::params![pitcher_id, season],
    )?;

    // Overall
    tx.execute(
        "INSERT INTO pitcher_profiles (pitcher_id, season, count_state, entropy, total_pitches)
         VALUES (?1, ?2, NULL, ?3, ?4)",
        rusqlite::params![pitcher_id, season, overall_entropy, total_pitches],
    )?;

    // Per count
    for ce in by_count {
        tx.execute(
            "INSERT INTO pitcher_profiles (pitcher_id, season, count_state, entropy, total_pitches)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![pitcher_id, season, ce.count_state, ce.entropy, ce.pitches],
        )?;
    }

    tx.commit()?;
    Ok(())
}
