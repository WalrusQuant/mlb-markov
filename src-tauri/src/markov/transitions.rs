use anyhow::Result;
use rusqlite::Connection;

use super::states::{encode_state, ALL_STATES};

pub struct TransitionMatrix {
    pub states: Vec<String>,
    pub matrix: Vec<Vec<f64>>,
    pub runs: Vec<Vec<f64>>,
    pub counts: Vec<Vec<i64>>,
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
        counts,
    })
}

fn normalize_matrix(counts: &[Vec<i64>], runs_total: &[Vec<f64>]) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let n = counts.len();
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

    let abs_idx = n - 1;
    matrix[abs_idx][abs_idx] = 1.0;

    (matrix, runs)
}

pub fn compute_momentum_transitions(
    conn: &Connection,
    season: i32,
    team_id: Option<i32>,
) -> Result<(TransitionMatrix, TransitionMatrix)> {
    let n = ALL_STATES.len();
    let mut cold_counts = vec![vec![0i64; n]; n];
    let mut cold_runs = vec![vec![0.0f64; n]; n];
    let mut hot_counts = vec![vec![0i64; n]; n];
    let mut hot_runs = vec![vec![0.0f64; n]; n];

    let team_filter = match team_id {
        Some(_) => "AND (g.home_team_id = ?2 OR g.away_team_id = ?2)",
        None => "",
    };

    let query = format!(
        "WITH play_context AS (
            SELECT p.outs_before, p.bases_before, p.outs_after, p.bases_after, p.runs_scored,
                   COALESCE(
                       SUM(p.runs_scored) OVER (
                           PARTITION BY p.game_pk, p.inning, p.half
                           ORDER BY p.id
                           ROWS BETWEEN UNBOUNDED PRECEDING AND 1 PRECEDING
                       ), 0
                   ) AS inning_runs_before
            FROM plays p
            JOIN games g ON g.game_pk = p.game_pk
            WHERE strftime('%Y', g.game_date) = ?1
              {}
        )
        SELECT outs_before, bases_before, outs_after, bases_after, runs_scored, inning_runs_before
        FROM play_context",
        team_filter
    );

    let mut stmt = conn.prepare(&query)?;

    let params: Vec<Box<dyn rusqlite::types::ToSql>> = match team_id {
        Some(tid) => vec![
            Box::new(season.to_string()),
            Box::new(tid as i64),
        ],
        None => vec![Box::new(season.to_string())],
    };
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut rows = stmt.query(param_refs.as_slice())?;

    while let Some(row) = rows.next()? {
        let outs_before: i32 = row.get(0)?;
        let bases_before: String = row.get(1)?;
        let outs_after: i32 = row.get(2)?;
        let bases_after: String = row.get(3)?;
        let runs_scored: i32 = row.get(4)?;
        let inning_runs_before: i64 = row.get(5)?;

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

        if inning_runs_before == 0 {
            cold_counts[from_idx][to_idx] += 1;
            cold_runs[from_idx][to_idx] += runs_scored as f64;
        } else {
            hot_counts[from_idx][to_idx] += 1;
            hot_runs[from_idx][to_idx] += runs_scored as f64;
        }
    }

    let (cold_matrix, cold_runs_avg) = normalize_matrix(&cold_counts, &cold_runs);
    let (hot_matrix, hot_runs_avg) = normalize_matrix(&hot_counts, &hot_runs);

    let states: Vec<String> = ALL_STATES.iter().map(|s| s.to_string()).collect();

    Ok((
        TransitionMatrix {
            states: states.clone(),
            matrix: cold_matrix,
            runs: cold_runs_avg,
            counts: cold_counts,
        },
        TransitionMatrix {
            states,
            matrix: hot_matrix,
            runs: hot_runs_avg,
            counts: hot_counts,
        },
    ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::schema::create_tables(&conn).unwrap();
        crate::db::schema::create_indexes(&conn).unwrap();
        conn
    }

    // -------------------------------------------------------------------------
    // normalize_matrix tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_normalize_matrix_simple() {
        // 3x3: row 0 has transitions, rows 1 and 2 are empty
        let counts = vec![
            vec![10i64, 20, 0],
            vec![0i64, 0, 0],
            vec![0i64, 0, 0],
        ];
        let runs_total = vec![
            vec![1.0f64, 4.0, 0.0],
            vec![0.0f64, 0.0, 0.0],
            vec![0.0f64, 0.0, 0.0],
        ];

        let (matrix, runs) = normalize_matrix(&counts, &runs_total);

        // Row 0: sum = 30, so [10/30, 20/30, 0] = [1/3, 2/3, 0]
        assert_abs_diff_eq!(matrix[0][0], 1.0 / 3.0, epsilon = 1e-10);
        assert_abs_diff_eq!(matrix[0][1], 2.0 / 3.0, epsilon = 1e-10);
        assert_abs_diff_eq!(matrix[0][2], 0.0, epsilon = 1e-10);

        // Row 1: all zeros (empty row, not the absorbing row)
        assert_abs_diff_eq!(matrix[1][0], 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(matrix[1][1], 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(matrix[1][2], 0.0, epsilon = 1e-10);

        // Row 2 (absorbing, index n-1): self-loop forced to 1.0
        assert_abs_diff_eq!(matrix[2][2], 1.0, epsilon = 1e-10);

        // Runs: avg = total / count for nonzero cells
        // runs[0][0] = 1.0 / 10 = 0.1, runs[0][1] = 4.0 / 20 = 0.2
        assert_abs_diff_eq!(runs[0][0], 0.1, epsilon = 1e-10);
        assert_abs_diff_eq!(runs[0][1], 0.2, epsilon = 1e-10);
        assert_abs_diff_eq!(runs[0][2], 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_normalize_matrix_row_stochastic() {
        let n = 25;
        let mut counts = vec![vec![0i64; n]; n];
        let mut runs_total = vec![vec![0.0f64; n]; n];

        // Put some transitions in the first three rows
        counts[0][1] = 5;
        counts[0][3] = 15;
        counts[1][0] = 1;
        counts[1][8] = 3;
        counts[1][24] = 6;
        counts[2][24] = 10;

        runs_total[0][1] = 2.0;
        runs_total[1][0] = 0.5;

        let (matrix, _runs) = normalize_matrix(&counts, &runs_total);

        // Every row with nonzero counts must sum to 1.0
        for i in 0..n {
            let row_sum: f64 = matrix[i].iter().sum();
            let row_counts_sum: i64 = counts[i].iter().sum();

            if i == n - 1 {
                // Absorbing state self-loop: always 1.0
                assert_abs_diff_eq!(row_sum, 1.0, epsilon = 1e-10);
            } else if row_counts_sum > 0 {
                assert_abs_diff_eq!(row_sum, 1.0, epsilon = 1e-10);
            } else {
                // Zero row: sums to 0 (no mass)
                assert_abs_diff_eq!(row_sum, 0.0, epsilon = 1e-10);
            }
        }
    }

    #[test]
    fn test_normalize_matrix_empty() {
        let n = 25;
        let counts = vec![vec![0i64; n]; n];
        let runs_total = vec![vec![0.0f64; n]; n];

        let (matrix, runs) = normalize_matrix(&counts, &runs_total);

        // All entries should be 0, except the absorbing self-loop
        for i in 0..n {
            for j in 0..n {
                if i == n - 1 && j == n - 1 {
                    assert_abs_diff_eq!(matrix[i][j], 1.0, epsilon = 1e-10);
                } else {
                    assert_abs_diff_eq!(matrix[i][j], 0.0, epsilon = 1e-10);
                }
                assert_abs_diff_eq!(runs[i][j], 0.0, epsilon = 1e-10);
            }
        }
    }

    // -------------------------------------------------------------------------
    // compute_offense_transitions tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_compute_offense_transitions_empty_db() {
        let conn = test_db();

        let result = compute_offense_transitions(&conn, 2024, None);
        assert!(result.is_ok(), "empty DB should succeed: {:?}", result.err());

        let tm = result.unwrap();

        // All counts must be zero
        for row in &tm.counts {
            for &c in row {
                assert_eq!(c, 0);
            }
        }

        // All matrix entries zero except absorbing self-loop
        let n = tm.matrix.len();
        for i in 0..n {
            for j in 0..n {
                if i == n - 1 && j == n - 1 {
                    assert_abs_diff_eq!(tm.matrix[i][j], 1.0, epsilon = 1e-10);
                } else {
                    assert_abs_diff_eq!(tm.matrix[i][j], 0.0, epsilon = 1e-10);
                }
            }
        }
    }

    #[test]
    fn test_compute_offense_transitions_single_play() {
        let conn = test_db();

        // Insert the two teams referenced by the game
        conn.execute(
            "INSERT INTO teams (team_id, name, abbreviation) VALUES (1, 'Team A', 'TMA')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO teams (team_id, name, abbreviation) VALUES (2, 'Team B', 'TMB')",
            [],
        )
        .unwrap();

        // Insert a game in 2024
        conn.execute(
            "INSERT INTO games (game_pk, game_date, home_team_id, away_team_id, status, data_fetched)
             VALUES (1, '2024-06-01', 1, 2, 'Final', 1)",
            [],
        )
        .unwrap();

        // Insert a single play: 0 outs, bases empty -> 0 outs, runner on first
        // ALL_STATES index 0 = "0_000", index 1 = "0_100"
        conn.execute(
            "INSERT INTO plays (game_pk, inning, half, event, outs_before, outs_after,
                                bases_before, bases_after, runs_scored)
             VALUES (1, 1, 'top', 'Single', 0, 0, '000', '100', 0)",
            [],
        )
        .unwrap();

        let tm = compute_offense_transitions(&conn, 2024, None).unwrap();

        // counts[0][1] must be 1
        assert_eq!(tm.counts[0][1], 1, "expected one transition from 0_000 to 0_100");

        // probability of that transition must be 1.0 (only transition from state 0)
        assert_abs_diff_eq!(tm.matrix[0][1], 1.0, epsilon = 1e-10);

        // No other non-absorbing transition should have count > 0
        let n = tm.counts.len();
        for i in 0..n {
            for j in 0..n {
                if i == 0 && j == 1 {
                    continue; // the expected transition
                }
                if i == n - 1 && j == n - 1 {
                    continue; // absorbing self-loop
                }
                assert_eq!(
                    tm.counts[i][j], 0,
                    "unexpected count at [{i}][{j}] = {}",
                    tm.counts[i][j]
                );
            }
        }
    }

    #[test]
    fn test_compute_offense_transitions_team_filter() {
        let conn = test_db();

        // Teams
        for (id, name, abbr) in [(1, "Team A", "TMA"), (2, "Team B", "TMB"), (3, "Team C", "TMC")] {
            conn.execute(
                "INSERT INTO teams (team_id, name, abbreviation) VALUES (?1, ?2, ?3)",
                rusqlite::params![id, name, abbr],
            )
            .unwrap();
        }

        // Game 1: teams 1 vs 2
        conn.execute(
            "INSERT INTO games (game_pk, game_date, home_team_id, away_team_id, status, data_fetched)
             VALUES (1, '2024-06-01', 1, 2, 'Final', 1)",
            [],
        )
        .unwrap();

        // Game 2: teams 2 vs 3 (team 1 not involved)
        conn.execute(
            "INSERT INTO games (game_pk, game_date, home_team_id, away_team_id, status, data_fetched)
             VALUES (2, '2024-06-02', 2, 3, 'Final', 1)",
            [],
        )
        .unwrap();

        // Play in game 1: 0_000 -> 0_100
        conn.execute(
            "INSERT INTO plays (game_pk, inning, half, event, outs_before, outs_after,
                                bases_before, bases_after, runs_scored)
             VALUES (1, 1, 'top', 'Single', 0, 0, '000', '100', 0)",
            [],
        )
        .unwrap();

        // Play in game 2: 0_000 -> 1_000 (strikeout)
        conn.execute(
            "INSERT INTO plays (game_pk, inning, half, event, outs_before, outs_after,
                                bases_before, bases_after, runs_scored)
             VALUES (2, 1, 'top', 'Strikeout', 0, 1, '000', '000', 0)",
            [],
        )
        .unwrap();

        // team_id = Some(1): should only see game-1 play (0_000 -> 0_100)
        let tm_team1 = compute_offense_transitions(&conn, 2024, Some(1)).unwrap();
        assert_eq!(tm_team1.counts[0][1], 1, "team 1 should see the single");
        assert_eq!(tm_team1.counts[0][8], 0, "team 1 should not see the strikeout from game 2");

        // team_id = None: should see both plays
        let tm_all = compute_offense_transitions(&conn, 2024, None).unwrap();
        assert_eq!(tm_all.counts[0][1], 1, "all-teams should see the single");
        assert_eq!(tm_all.counts[0][8], 1, "all-teams should see the strikeout");
    }

    #[test]
    fn test_compute_offense_transitions_stores_to_db() {
        let conn = test_db();

        // Minimal data: one team, one game, one play
        conn.execute(
            "INSERT INTO teams (team_id, name, abbreviation) VALUES (1, 'Team A', 'TMA')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO games (game_pk, game_date, home_team_id, away_team_id, status, data_fetched)
             VALUES (1, '2024-07-04', 1, 1, 'Final', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO plays (game_pk, inning, half, event, outs_before, outs_after,
                                bases_before, bases_after, runs_scored)
             VALUES (1, 1, 'top', 'Double', 0, 0, '000', '010', 0)",
            [],
        )
        .unwrap();

        compute_offense_transitions(&conn, 2024, None).unwrap();

        // The store should have persisted at least the one real transition
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM offense_transitions WHERE season = 2024 AND team_id IS NULL",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!(count >= 1, "offense_transitions should have at least one row after compute");

        // Verify the specific transition (0_000 -> 0_010) was stored with correct values
        let row: (i64, f64) = conn
            .query_row(
                "SELECT occurrences, probability
                 FROM offense_transitions
                 WHERE season = 2024
                   AND team_id IS NULL
                   AND state_from = '0_000'
                   AND state_to   = '0_010'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(row.0, 1, "stored occurrences should be 1");
        assert_abs_diff_eq!(row.1, 1.0, epsilon = 1e-10);
    }
}
