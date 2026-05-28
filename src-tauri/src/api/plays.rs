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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    // -----------------------------------------------------------------------
    // Helper: encode_bases
    // -----------------------------------------------------------------------

    #[test]
    fn test_encode_bases_empty() {
        assert_eq!(encode_bases(&[false, false, false]), "000");
    }

    #[test]
    fn test_encode_bases_full() {
        assert_eq!(encode_bases(&[true, true, true]), "111");
    }

    #[test]
    fn test_encode_bases_mixed() {
        assert_eq!(encode_bases(&[true, false, true]), "101");
    }

    // -----------------------------------------------------------------------
    // Helper: base_index
    // -----------------------------------------------------------------------

    #[test]
    fn test_base_index_valid() {
        assert_eq!(base_index("1B"), Some(0));
        assert_eq!(base_index("2B"), Some(1));
        assert_eq!(base_index("3B"), Some(2));
    }

    #[test]
    fn test_base_index_invalid() {
        assert_eq!(base_index("score"), None);
        assert_eq!(base_index(""), None);
        assert_eq!(base_index("home"), None);
    }

    // -----------------------------------------------------------------------
    // parse_game via JSON deserialization
    // -----------------------------------------------------------------------

    /// Deserialize a JSON string into PlayByPlayResponse, panicking on failure.
    fn parse_json(json: &str) -> PlayByPlayResponse {
        serde_json::from_str(json).expect("test JSON must deserialize cleanly")
    }

    #[test]
    fn test_parse_game_strikeout() {
        let raw = parse_json(r#"{
            "allPlays": [{
                "about": {
                    "atBatIndex": 0,
                    "halfInning": "top",
                    "inning": 1,
                    "isComplete": true
                },
                "result": { "event": "Strikeout", "eventType": "strikeout" },
                "matchup": {
                    "batter":    { "id": 1, "fullName": "Batter" },
                    "batSide":   { "code": "R" },
                    "pitcher":   { "id": 2, "fullName": "Pitcher" },
                    "pitchHand": { "code": "R" }
                },
                "count": { "balls": 1, "strikes": 3, "outs": 1 },
                "runners": [{
                    "movement": { "start": null, "end": null, "isOut": true },
                    "details":  { "runner": { "id": 1, "fullName": "Batter" } }
                }],
                "playEvents": []
            }]
        }"#);

        let game = parse_game(1, raw);
        assert_eq!(game.plays.len(), 1);

        let (play, pitches) = &game.plays[0];
        assert_eq!(play.outs_before, 0);
        assert_eq!(play.outs_after, 1);
        assert_eq!(play.bases_before, "000");
        assert_eq!(play.bases_after, "000");
        assert_eq!(play.runs_scored, 0);
        assert!(pitches.is_empty());
    }

    #[test]
    fn test_parse_game_single_runner_scores() {
        // Play 1: Double — batter moves to 2B. Sets current_bases to [false, true, false].
        // Play 2: Single — batter moves to 1B, runner from 2B scores.
        //   Expected for play 2: bases_before="010", bases_after="100", runs_scored=1.
        let raw = parse_json(r#"{
            "allPlays": [
                {
                    "about": {
                        "atBatIndex": 0,
                        "halfInning": "top",
                        "inning": 1,
                        "isComplete": true
                    },
                    "result": { "event": "Double", "eventType": "double" },
                    "matchup": {
                        "batter":    { "id": 10, "fullName": "Batter A" },
                        "batSide":   { "code": "R" },
                        "pitcher":   { "id": 20, "fullName": "Pitcher A" },
                        "pitchHand": { "code": "R" }
                    },
                    "count": { "balls": 0, "strikes": 0, "outs": 0 },
                    "runners": [{
                        "movement": { "start": null, "end": "2B", "isOut": false },
                        "details":  { "runner": { "id": 10, "fullName": "Batter A" } }
                    }],
                    "playEvents": []
                },
                {
                    "about": {
                        "atBatIndex": 1,
                        "halfInning": "top",
                        "inning": 1,
                        "isComplete": true
                    },
                    "result": { "event": "Single", "eventType": "single" },
                    "matchup": {
                        "batter":    { "id": 11, "fullName": "Batter B" },
                        "batSide":   { "code": "L" },
                        "pitcher":   { "id": 20, "fullName": "Pitcher A" },
                        "pitchHand": { "code": "R" }
                    },
                    "count": { "balls": 0, "strikes": 0, "outs": 0 },
                    "runners": [
                        {
                            "movement": { "start": null, "end": "1B", "isOut": false },
                            "details":  { "runner": { "id": 11, "fullName": "Batter B" } }
                        },
                        {
                            "movement": { "start": "2B", "end": "score", "isOut": false },
                            "details":  { "runner": { "id": 10, "fullName": "Batter A" } }
                        }
                    ],
                    "playEvents": []
                }
            ]
        }"#);

        let game = parse_game(1, raw);
        assert_eq!(game.plays.len(), 2);

        let (play2, _) = &game.plays[1];
        assert_eq!(play2.bases_before, "010");
        assert_eq!(play2.bases_after, "100");
        assert_eq!(play2.runs_scored, 1);
    }

    #[test]
    fn test_parse_game_half_inning_reset() {
        // Play 1: top of 1st — single, batter reaches 1B → bases become "100".
        // Play 2: bottom of 1st — bases_before must be "000" (reset on half-inning change).
        let raw = parse_json(r#"{
            "allPlays": [
                {
                    "about": {
                        "atBatIndex": 0,
                        "halfInning": "top",
                        "inning": 1,
                        "isComplete": true
                    },
                    "result": { "event": "Single", "eventType": "single" },
                    "matchup": {
                        "batter":    { "id": 1, "fullName": "Batter" },
                        "batSide":   { "code": "R" },
                        "pitcher":   { "id": 2, "fullName": "Pitcher" },
                        "pitchHand": { "code": "R" }
                    },
                    "count": { "balls": 0, "strikes": 0, "outs": 0 },
                    "runners": [{
                        "movement": { "start": null, "end": "1B", "isOut": false },
                        "details":  { "runner": { "id": 1, "fullName": "Batter" } }
                    }],
                    "playEvents": []
                },
                {
                    "about": {
                        "atBatIndex": 1,
                        "halfInning": "bottom",
                        "inning": 1,
                        "isComplete": true
                    },
                    "result": { "event": "Strikeout", "eventType": "strikeout" },
                    "matchup": {
                        "batter":    { "id": 3, "fullName": "Batter 2" },
                        "batSide":   { "code": "R" },
                        "pitcher":   { "id": 4, "fullName": "Pitcher 2" },
                        "pitchHand": { "code": "R" }
                    },
                    "count": { "balls": 0, "strikes": 3, "outs": 1 },
                    "runners": [{
                        "movement": { "start": null, "end": null, "isOut": true },
                        "details":  { "runner": { "id": 3, "fullName": "Batter 2" } }
                    }],
                    "playEvents": []
                }
            ]
        }"#);

        let game = parse_game(1, raw);
        assert_eq!(game.plays.len(), 2);

        let (play1, _) = &game.plays[0];
        assert_eq!(play1.bases_after, "100");

        let (play2, _) = &game.plays[1];
        assert_eq!(play2.bases_before, "000");
    }

    #[test]
    fn test_parse_game_three_outs_reset() {
        // Play 1 (top 1st): Single — runner on 1B. bases_after="100".
        // Play 2 (top 1st): Triple play — 3 outs recorded with the runner on 1B
        //   still present (bases_after still shows "001" from the triple advance,
        //   but internal current_bases is cleared by the outs_after >= 3 branch).
        // Play 3 (bottom 1st): bases_before must be "000" (cleared by both the
        //   3-out reset AND the half-inning change — belt and suspenders).
        //   This confirms the 3-out reset path ran: without it the internal state
        //   would carry "001" into the half-inning check, which would clear it
        //   anyway, but the 3-out branch is the primary guard.
        let raw = parse_json(r#"{
            "allPlays": [
                {
                    "about": {
                        "atBatIndex": 0,
                        "halfInning": "top",
                        "inning": 1,
                        "isComplete": true
                    },
                    "result": { "event": "Single", "eventType": "single" },
                    "matchup": {
                        "batter":    { "id": 1, "fullName": "Batter A" },
                        "batSide":   { "code": "R" },
                        "pitcher":   { "id": 2, "fullName": "Pitcher" },
                        "pitchHand": { "code": "R" }
                    },
                    "count": { "balls": 0, "strikes": 0, "outs": 0 },
                    "runners": [{
                        "movement": { "start": null, "end": "1B", "isOut": false },
                        "details":  { "runner": { "id": 1, "fullName": "Batter A" } }
                    }],
                    "playEvents": []
                },
                {
                    "about": {
                        "atBatIndex": 1,
                        "halfInning": "top",
                        "inning": 1,
                        "isComplete": true
                    },
                    "result": { "event": "Triple Play", "eventType": "triple_play" },
                    "matchup": {
                        "batter":    { "id": 3, "fullName": "Batter B" },
                        "batSide":   { "code": "R" },
                        "pitcher":   { "id": 2, "fullName": "Pitcher" },
                        "pitchHand": { "code": "R" }
                    },
                    "count": { "balls": 0, "strikes": 0, "outs": 3 },
                    "runners": [
                        {
                            "movement": { "start": null, "end": null, "isOut": true },
                            "details":  { "runner": { "id": 3, "fullName": "Batter B" } }
                        },
                        {
                            "movement": { "start": "1B", "end": null, "isOut": true },
                            "details":  { "runner": { "id": 1, "fullName": "Batter A" } }
                        },
                        {
                            "movement": { "start": null, "end": null, "isOut": true },
                            "details":  { "runner": { "id": 5, "fullName": "Runner C" } }
                        }
                    ],
                    "playEvents": []
                },
                {
                    "about": {
                        "atBatIndex": 2,
                        "halfInning": "bottom",
                        "inning": 1,
                        "isComplete": true
                    },
                    "result": { "event": "Strikeout", "eventType": "strikeout" },
                    "matchup": {
                        "batter":    { "id": 6, "fullName": "Batter C" },
                        "batSide":   { "code": "L" },
                        "pitcher":   { "id": 7, "fullName": "Pitcher 2" },
                        "pitchHand": { "code": "L" }
                    },
                    "count": { "balls": 0, "strikes": 3, "outs": 1 },
                    "runners": [{
                        "movement": { "start": null, "end": null, "isOut": true },
                        "details":  { "runner": { "id": 6, "fullName": "Batter C" } }
                    }],
                    "playEvents": []
                }
            ]
        }"#);

        let game = parse_game(1, raw);
        assert_eq!(game.plays.len(), 3);

        // Play 1 ends with a runner on 1B.
        let (play1, _) = &game.plays[0];
        assert_eq!(play1.bases_after, "100");

        // Play 2 starts with that runner on 1B; the triple play clears the bases.
        let (play2, _) = &game.plays[1];
        assert_eq!(play2.bases_before, "100");
        assert_eq!(play2.outs_after, 3);

        // Play 3 (new half-inning) starts clean — the 3-out reset and half-inning
        // reset both guarantee this.
        let (play3, _) = &game.plays[2];
        assert_eq!(play3.bases_before, "000");
    }

    #[test]
    fn test_parse_game_null_isout() {
        // "isOut": null should deserialize as false (custom deserializer).
        // "isOut" omitted entirely should also be false (serde default).
        let raw = parse_json(r#"{
            "allPlays": [
                {
                    "about": {
                        "atBatIndex": 0,
                        "halfInning": "top",
                        "inning": 1,
                        "isComplete": true
                    },
                    "result": { "event": "Single", "eventType": "single" },
                    "matchup": {
                        "batter":    { "id": 1, "fullName": "Batter" },
                        "batSide":   { "code": "R" },
                        "pitcher":   { "id": 2, "fullName": "Pitcher" },
                        "pitchHand": { "code": "R" }
                    },
                    "count": { "balls": 0, "strikes": 0, "outs": 0 },
                    "runners": [
                        {
                            "movement": { "start": null, "end": "1B", "isOut": null },
                            "details":  { "runner": { "id": 1, "fullName": "Batter" } }
                        }
                    ],
                    "playEvents": []
                },
                {
                    "about": {
                        "atBatIndex": 1,
                        "halfInning": "top",
                        "inning": 1,
                        "isComplete": true
                    },
                    "result": { "event": "Single", "eventType": "single" },
                    "matchup": {
                        "batter":    { "id": 3, "fullName": "Batter 2" },
                        "batSide":   { "code": "R" },
                        "pitcher":   { "id": 2, "fullName": "Pitcher" },
                        "pitchHand": { "code": "R" }
                    },
                    "count": { "balls": 0, "strikes": 0, "outs": 0 },
                    "runners": [
                        {
                            "movement": { "start": null, "end": "2B" },
                            "details":  { "runner": { "id": 3, "fullName": "Batter 2" } }
                        },
                        {
                            "movement": { "start": "1B", "end": "3B" },
                            "details":  { "runner": { "id": 1, "fullName": "Batter" } }
                        }
                    ],
                    "playEvents": []
                }
            ]
        }"#);

        let game = parse_game(1, raw);
        assert_eq!(game.plays.len(), 2);

        // Play 1: isOut=null means the batter reached base safely — outs_before=0,
        // outs_after=0, runner ends up on 1B.
        let (play1, _) = &game.plays[0];
        assert_eq!(play1.outs_before, 0);
        assert_eq!(play1.outs_after, 0);
        assert_eq!(play1.bases_after, "100");

        // Play 2: isOut omitted entirely — both runners treated as safe.
        // Runner from 1B advances to 3B; batter goes to 2B.
        // bases_before="100", bases_after="011"
        let (play2, _) = &game.plays[1];
        assert_eq!(play2.bases_before, "100");
        assert_eq!(play2.outs_before, 0);
        assert_eq!(play2.outs_after, 0);
        assert_eq!(play2.bases_after, "011");
    }
}
