use anyhow::Result;
use reqwest::Client;
use rusqlite::Connection;
use serde::Deserialize;

use super::client::BASE_URL;

#[derive(Debug, Deserialize)]
struct ApiScheduleResponse {
    #[serde(default)]
    dates: Vec<ApiDate>,
}

#[derive(Debug, Deserialize)]
struct ApiDate {
    #[serde(default)]
    games: Vec<ApiGame>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiGame {
    game_pk: i64,
    official_date: Option<String>,
    #[serde(default)]
    status: ApiStatus,
    #[serde(default)]
    teams: ApiTeams,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiStatus {
    #[serde(default)]
    detailed_state: String,
}

#[derive(Debug, Default, Deserialize)]
struct ApiTeams {
    #[serde(default)]
    home: ApiTeamWrapper,
    #[serde(default)]
    away: ApiTeamWrapper,
}

#[derive(Debug, Default, Deserialize)]
struct ApiTeamWrapper {
    #[serde(default)]
    team: ApiTeam,
}

#[derive(Debug, Default, Deserialize)]
struct ApiTeam {
    #[serde(default)]
    id: i64,
    #[serde(default)]
    name: String,
}

pub struct GameInfo {
    pub game_pk: i64,
    pub game_date: String,
    pub home_team_id: i64,
    pub home_team_name: String,
    pub away_team_id: i64,
    pub away_team_name: String,
    pub status: String,
}

pub async fn fetch_schedule(client: &Client, season: i32) -> Result<Vec<GameInfo>> {
    let url = format!(
        "{}/schedule?sportId=1&season={}&gameType=R&hydrate=team",
        BASE_URL, season
    );
    let resp: ApiScheduleResponse = client.get(&url).send().await?.json().await?;

    let mut games = Vec::new();
    for date in resp.dates {
        for g in date.games {
            let game_date = g.official_date.unwrap_or_default();
            if game_date.is_empty() {
                continue;
            }
            games.push(GameInfo {
                game_pk: g.game_pk,
                game_date,
                home_team_id: g.teams.home.team.id,
                home_team_name: g.teams.home.team.name.clone(),
                away_team_id: g.teams.away.team.id,
                away_team_name: g.teams.away.team.name.clone(),
                status: g.status.detailed_state,
            });
        }
    }
    Ok(games)
}

pub fn upsert_schedule(conn: &Connection, games: &[GameInfo]) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    let mut team_set = std::collections::HashSet::new();
    for g in games {
        team_set.insert((g.home_team_id, g.home_team_name.clone()));
        team_set.insert((g.away_team_id, g.away_team_name.clone()));
    }
    for (id, name) in &team_set {
        tx.execute(
            "INSERT INTO teams (team_id, name) VALUES (?1, ?2)
             ON CONFLICT(team_id) DO UPDATE SET name = excluded.name, updated_at = datetime('now')",
            rusqlite::params![id, name],
        )?;
    }

    for g in games {
        tx.execute(
            "INSERT INTO games (game_pk, game_date, home_team_id, away_team_id, status)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(game_pk) DO UPDATE SET
               status = excluded.status,
               home_team_id = excluded.home_team_id,
               away_team_id = excluded.away_team_id",
            rusqlite::params![
                g.game_pk,
                g.game_date,
                g.home_team_id,
                g.away_team_id,
                g.status,
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}
