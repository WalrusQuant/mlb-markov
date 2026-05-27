use anyhow::Result;
use rusqlite::Connection;
use std::collections::HashMap;

pub struct PitchMatrix {
    pub types: Vec<String>,
    pub matrix: Vec<Vec<f64>>,
    pub occurrences: Vec<Vec<i64>>,
}

pub struct PitchTransitionSet {
    pub pitcher_id: i64,
    pub season: i32,
    pub pitch_types: Vec<String>,
    pub overall: PitchMatrix,
    pub by_count: HashMap<String, PitchMatrix>,
}

pub fn compute_pitch_transitions(
    conn: &Connection,
    pitcher_id: i64,
    season: i32,
) -> Result<PitchTransitionSet> {
    // Get consecutive pitch pairs within the same at-bat for this pitcher
    let mut stmt = conn.prepare(
        "SELECT p1.pitch_type, p2.pitch_type,
                printf('%d-%d', p2.count_balls, p2.count_strikes)
         FROM pitches p1
         JOIN pitches p2 ON p1.play_id = p2.play_id AND p2.pitch_number = p1.pitch_number + 1
         JOIN plays pl ON pl.id = p1.play_id
         JOIN games g ON g.game_pk = pl.game_pk
         WHERE pl.pitcher_id = ?1
           AND strftime('%Y', g.game_date) = ?2
           AND p1.pitch_type != ''
           AND p2.pitch_type != ''",
    )?;

    // Collect all transitions grouped by count state
    let mut overall_counts: HashMap<(String, String), i64> = HashMap::new();
    let mut by_count_counts: HashMap<String, HashMap<(String, String), i64>> = HashMap::new();
    let mut all_types: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    let mut rows = stmt.query(rusqlite::params![pitcher_id, season.to_string()])?;
    while let Some(row) = rows.next()? {
        let from: String = row.get(0)?;
        let to: String = row.get(1)?;
        let count_state: String = row.get(2)?;

        all_types.insert(from.clone());
        all_types.insert(to.clone());

        *overall_counts.entry((from.clone(), to.clone())).or_default() += 1;
        *by_count_counts
            .entry(count_state)
            .or_default()
            .entry((from, to))
            .or_default() += 1;
    }

    let pitch_types: Vec<String> = all_types.into_iter().collect();

    let overall = build_matrix(&pitch_types, &overall_counts);

    let mut by_count = HashMap::new();
    for (cs, transitions) in &by_count_counts {
        by_count.insert(cs.clone(), build_matrix(&pitch_types, transitions));
    }

    // Store in DB
    store_pitch_transitions(conn, pitcher_id, season, &pitch_types, &overall, &by_count)?;

    Ok(PitchTransitionSet {
        pitcher_id,
        season,
        pitch_types,
        overall,
        by_count,
    })
}

fn build_matrix(
    types: &[String],
    counts: &HashMap<(String, String), i64>,
) -> PitchMatrix {
    let n = types.len();
    let type_idx: HashMap<&str, usize> = types.iter().enumerate().map(|(i, t)| (t.as_str(), i)).collect();

    let mut occ = vec![vec![0i64; n]; n];
    let mut matrix = vec![vec![0.0f64; n]; n];

    for ((from, to), &count) in counts {
        if let (Some(&fi), Some(&ti)) = (type_idx.get(from.as_str()), type_idx.get(to.as_str())) {
            occ[fi][ti] = count;
        }
    }

    // Row-normalize
    for i in 0..n {
        let row_sum: i64 = occ[i].iter().sum();
        if row_sum > 0 {
            for j in 0..n {
                matrix[i][j] = occ[i][j] as f64 / row_sum as f64;
            }
        }
    }

    PitchMatrix {
        types: types.to_vec(),
        matrix,
        occurrences: occ,
    }
}

fn store_pitch_transitions(
    conn: &Connection,
    pitcher_id: i64,
    season: i32,
    types: &[String],
    overall: &PitchMatrix,
    by_count: &HashMap<String, PitchMatrix>,
) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    tx.execute(
        "DELETE FROM pitch_transitions WHERE pitcher_id = ?1 AND season = ?2",
        rusqlite::params![pitcher_id, season],
    )?;

    let n = types.len();

    // Overall (count_state = NULL)
    for i in 0..n {
        for j in 0..n {
            if overall.occurrences[i][j] == 0 {
                continue;
            }
            tx.execute(
                "INSERT INTO pitch_transitions
                 (pitcher_id, season, count_state, pitch_from, pitch_to, occurrences, probability)
                 VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    pitcher_id,
                    season,
                    types[i],
                    types[j],
                    overall.occurrences[i][j],
                    overall.matrix[i][j],
                ],
            )?;
        }
    }

    // Per count state
    for (cs, mat) in by_count {
        for i in 0..n {
            for j in 0..n {
                if mat.occurrences[i][j] == 0 {
                    continue;
                }
                tx.execute(
                    "INSERT INTO pitch_transitions
                     (pitcher_id, season, count_state, pitch_from, pitch_to, occurrences, probability)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    rusqlite::params![
                        pitcher_id,
                        season,
                        cs,
                        types[i],
                        types[j],
                        mat.occurrences[i][j],
                        mat.matrix[i][j],
                    ],
                )?;
            }
        }
    }

    tx.commit()?;
    Ok(())
}
