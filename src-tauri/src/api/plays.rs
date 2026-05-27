use anyhow::Result;
use reqwest::Client;
use rusqlite::Connection;
use serde::Deserialize;

use super::client::BASE_URL;

// ---------------------------------------------------------------------------
// API DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayByPlayResponse {
    #[serde(default)]
    all_plays: Vec<ApiPlay>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiPlay {
    #[serde(default)]
    about: ApiAbout,
    #[serde(default)]
    result: ApiResult,
    #[serde(default)]
    matchup: ApiMatchup,
    #[serde(default)]
    count: ApiCount,
    #[serde(default)]
    runners: Vec<ApiRunner>,
    #[serde(default)]
    play_events: Vec<ApiPlayEvent>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiAbout {
    #[serde(default)]
    at_bat_index: i32,
    #[serde(default)]
    half_inning: String,
    #[serde(default)]
    inning: i32,
    #[serde(default)]
    is_complete: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiResult {
    #[serde(default)]
    event: String,
    #[serde(default)]
    event_type: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiMatchup {
    #[serde(default)]
    batter: ApiPerson,
    #[serde(default)]
    bat_side: ApiSide,
    #[serde(default)]
    pitcher: ApiPerson,
    #[serde(default)]
    pitch_hand: ApiSide,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiPerson {
    #[serde(default)]
    id: i64,
    #[serde(default)]
    full_name: String,
}

#[derive(Debug, Default, Deserialize)]
struct ApiSide {
    #[serde(default)]
    code: String,
}

#[derive(Debug, Default, Deserialize)]
struct ApiCount {
    #[serde(default)]
    balls: i32,
    #[serde(default)]
    strikes: i32,
    #[serde(default)]
    outs: i32,
}

#[derive(Debug, Deserialize)]
struct ApiRunner {
    #[serde(default)]
    movement: ApiMovement,
    #[serde(default)]
    details: ApiRunnerDetails,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiMovement {
    start: Option<String>,
    end: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_bool")]
    is_out: bool,
}

fn deserialize_null_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<bool>::deserialize(deserializer).map(|opt| opt.unwrap_or(false))
}

#[derive(Debug, Default, Deserialize)]
struct ApiRunnerDetails {
    #[serde(default)]
    runner: ApiPerson,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiPlayEvent {
    #[serde(default)]
    is_pitch: bool,
    #[serde(default)]
    details: ApiEventDetails,
    #[serde(default)]
    count: ApiCount,
    #[serde(default)]
    pitch_data: Option<ApiPitchData>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiEventDetails {
    #[serde(default)]
    description: String,
    #[serde(rename = "type")]
    pitch_type: Option<ApiPitchType>,
}

#[derive(Debug, Default, Deserialize)]
struct ApiPitchType {
    #[serde(default)]
    code: String,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiPitchData {
    start_speed: Option<f64>,
}

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

pub struct PlayRecord {
    pub game_pk: i64,
    pub inning: i32,
    pub half: String,
    pub event: String,
    pub outs_before: i32,
    pub outs_after: i32,
    pub bases_before: String,
    pub bases_after: String,
    pub runs_scored: i32,
    pub batter_id: Option<i64>,
    pub pitcher_id: Option<i64>,
}

pub struct PitchRecord {
    pub pitch_number: i32,
    pub pitch_type: String,
    pub pitch_type_desc: String,
    pub release_speed: Option<f64>,
    pub count_balls: i32,
    pub count_strikes: i32,
    pub result: String,
}

pub struct PlayerRecord {
    pub player_id: i64,
    pub full_name: String,
    pub throws: Option<String>,
    pub bats: Option<String>,
}

pub struct ParsedGame {
    pub plays: Vec<(PlayRecord, Vec<PitchRecord>)>,
    pub players: Vec<PlayerRecord>,
}

// ---------------------------------------------------------------------------
// Fetch + parse
// ---------------------------------------------------------------------------

pub async fn fetch_play_by_play(client: &Client, game_pk: i64) -> Result<PlayByPlayResponse> {
    let url = format!("{}/game/{}/playByPlay", BASE_URL, game_pk);
    let resp: PlayByPlayResponse = client.get(&url).send().await?.json().await?;
    Ok(resp)
}

pub fn parse_game(game_pk: i64, raw: PlayByPlayResponse) -> ParsedGame {
    let mut result_plays = Vec::new();
    let mut players = Vec::new();
    let mut seen_players = std::collections::HashSet::new();

    // Track base state across plays within each half-inning
    let mut current_bases = [false; 3]; // [1B, 2B, 3B]
    let mut current_half = String::new();
    let mut current_inning = 0i32;

    for play in &raw.all_plays {
        if !play.about.is_complete {
            continue;
        }
        if play.result.event.is_empty() {
            continue;
        }

        // Reset base state on half-inning change
        if play.about.inning != current_inning || play.about.half_inning != current_half {
            current_bases = [false; 3];
            current_inning = play.about.inning;
            current_half = play.about.half_inning.clone();
        }

        // Collect players
        let batter_id = play.matchup.batter.id;
        let pitcher_id = play.matchup.pitcher.id;
        if batter_id != 0 && seen_players.insert(batter_id) {
            players.push(PlayerRecord {
                player_id: batter_id,
                full_name: play.matchup.batter.full_name.clone(),
                bats: if play.matchup.bat_side.code.is_empty() {
                    None
                } else {
                    Some(play.matchup.bat_side.code.clone())
                },
                throws: None,
            });
        }
        if pitcher_id != 0 && seen_players.insert(pitcher_id) {
            players.push(PlayerRecord {
                player_id: pitcher_id,
                full_name: play.matchup.pitcher.full_name.clone(),
                throws: if play.matchup.pitch_hand.code.is_empty() {
                    None
                } else {
                    Some(play.matchup.pitch_hand.code.clone())
                },
                bats: None,
            });
        }

        // Compute outs
        let outs_made: i32 = play
            .runners
            .iter()
            .filter(|r| r.movement.is_out)
            .count() as i32;
        let outs_after = play.count.outs;
        let raw_outs_before = outs_after - outs_made;
        if raw_outs_before < 0 {
            eprintln!(
                "[mlb-markov] Negative outs_before ({}) for game {} at_bat {}, clamping to 0",
                raw_outs_before, game_pk, play.about.at_bat_index
            );
        }
        let outs_before = raw_outs_before.max(0);

        // Record bases_before from tracked state
        let bases_before = encode_bases(&current_bases);

        // Apply runner movements to get bases_after
        // First: remove runners who moved from their start positions
        for runner in &play.runners {
            if let Some(ref start) = runner.movement.start {
                if let Some(idx) = base_index(start) {
                    current_bases[idx] = false;
                }
            }
        }
        // Then: place runners at their end positions
        for runner in &play.runners {
            if let Some(ref end) = runner.movement.end {
                if let Some(idx) = base_index(end) {
                    current_bases[idx] = true;
                }
                // "score" and null don't set any base
            }
        }
        let bases_after = encode_bases(&current_bases);

        // Count runs scored
        let runs_scored = play
            .runners
            .iter()
            .filter(|r| r.movement.end.as_deref() == Some("score"))
            .count() as i32;

        // Reset bases if 3 outs
        if outs_after >= 3 {
            current_bases = [false; 3];
        }

        // Extract pitches
        let pitches: Vec<PitchRecord> = play
            .play_events
            .iter()
            .filter(|e| e.is_pitch)
            .enumerate()
            .map(|(i, e)| {
                let (pt_code, pt_desc) = match &e.details.pitch_type {
                    Some(pt) => (pt.code.clone(), pt.description.clone()),
                    None => (String::new(), String::new()),
                };
                PitchRecord {
                    pitch_number: (i + 1) as i32,
                    pitch_type: pt_code,
                    pitch_type_desc: pt_desc,
                    release_speed: e.pitch_data.as_ref().and_then(|pd| pd.start_speed),
                    count_balls: e.count.balls,
                    count_strikes: e.count.strikes,
                    result: e.details.description.clone(),
                }
            })
            .collect();

        let play_record = PlayRecord {
            game_pk,
            inning: play.about.inning,
            half: play.about.half_inning.clone(),
            event: play.result.event.clone(),
            outs_before,
            outs_after,
            bases_before,
            bases_after,
            runs_scored,
            batter_id: if batter_id == 0 { None } else { Some(batter_id) },
            pitcher_id: if pitcher_id == 0 { None } else { Some(pitcher_id) },
        };

        result_plays.push((play_record, pitches));
    }

    ParsedGame {
        plays: result_plays,
        players,
    }
}

// ---------------------------------------------------------------------------
// DB insertion
// ---------------------------------------------------------------------------

pub fn insert_parsed_game(conn: &Connection, game_pk: i64, parsed: &ParsedGame) -> Result<(i64, i64)> {
    let tx = conn.unchecked_transaction()?;
    let mut plays_inserted = 0i64;
    let mut pitches_inserted = 0i64;

    // Upsert players
    for p in &parsed.players {
        tx.execute(
            "INSERT INTO players (player_id, full_name, throws, bats)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(player_id) DO UPDATE SET
               full_name = excluded.full_name,
               throws = COALESCE(excluded.throws, players.throws),
               bats = COALESCE(excluded.bats, players.bats),
               updated_at = datetime('now')",
            rusqlite::params![p.player_id, p.full_name, p.throws, p.bats],
        )?;
    }

    // Insert plays + pitches
    for (play, pitches) in &parsed.plays {
        tx.execute(
            "INSERT INTO plays (game_pk, inning, half, event, outs_before, outs_after,
             bases_before, bases_after, runs_scored, batter_id, pitcher_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                play.game_pk,
                play.inning,
                play.half,
                play.event,
                play.outs_before,
                play.outs_after,
                play.bases_before,
                play.bases_after,
                play.runs_scored,
                play.batter_id,
                play.pitcher_id,
            ],
        )?;
        let play_id = tx.last_insert_rowid();
        plays_inserted += 1;

        for pitch in pitches {
            tx.execute(
                "INSERT INTO pitches (play_id, pitch_number, pitch_type, pitch_type_desc,
                 release_speed, count_balls, count_strikes, result)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    play_id,
                    pitch.pitch_number,
                    pitch.pitch_type,
                    pitch.pitch_type_desc,
                    pitch.release_speed,
                    pitch.count_balls,
                    pitch.count_strikes,
                    pitch.result,
                ],
            )?;
            pitches_inserted += 1;
        }
    }

    tx.execute(
        "UPDATE games SET data_fetched = 1 WHERE game_pk = ?1",
        rusqlite::params![game_pk],
    )?;

    tx.commit()?;
    Ok((plays_inserted, pitches_inserted))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn encode_bases(bases: &[bool; 3]) -> String {
    format!(
        "{}{}{}",
        if bases[0] { '1' } else { '0' },
        if bases[1] { '1' } else { '0' },
        if bases[2] { '1' } else { '0' },
    )
}

fn base_index(base: &str) -> Option<usize> {
    match base {
        "1B" => Some(0),
        "2B" => Some(1),
        "3B" => Some(2),
        _ => None,
    }
}
