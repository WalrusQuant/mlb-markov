use rusqlite::Connection;

pub fn create_tables(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS teams (
            team_id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            abbreviation TEXT NOT NULL DEFAULT '',
            league TEXT,
            division TEXT,
            updated_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS players (
            player_id INTEGER PRIMARY KEY,
            full_name TEXT NOT NULL,
            primary_position TEXT,
            throws TEXT,
            bats TEXT,
            team_id INTEGER REFERENCES teams(team_id),
            updated_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS plays (
            id INTEGER PRIMARY KEY,
            game_pk INTEGER NOT NULL,
            inning INTEGER NOT NULL,
            half TEXT NOT NULL,
            event TEXT NOT NULL,
            outs_before INTEGER,
            outs_after INTEGER,
            bases_before TEXT,
            bases_after TEXT,
            runs_scored INTEGER DEFAULT 0,
            batter_id INTEGER REFERENCES players(player_id),
            pitcher_id INTEGER REFERENCES players(player_id),
            created_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS pitches (
            id INTEGER PRIMARY KEY,
            play_id INTEGER REFERENCES plays(id),
            pitch_number INTEGER,
            pitch_type TEXT,
            pitch_type_desc TEXT,
            release_speed REAL,
            count_balls INTEGER,
            count_strikes INTEGER,
            result TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS offense_transitions (
            id INTEGER PRIMARY KEY,
            season INTEGER NOT NULL,
            team_id INTEGER REFERENCES teams(team_id),
            state_from TEXT NOT NULL,
            state_to TEXT NOT NULL,
            occurrences INTEGER NOT NULL,
            probability REAL NOT NULL,
            avg_runs_scored REAL DEFAULT 0,
            updated_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS pitch_transitions (
            id INTEGER PRIMARY KEY,
            pitcher_id INTEGER NOT NULL REFERENCES players(player_id),
            season INTEGER NOT NULL,
            count_state TEXT,
            pitch_from TEXT NOT NULL,
            pitch_to TEXT NOT NULL,
            occurrences INTEGER NOT NULL,
            probability REAL NOT NULL,
            updated_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS pitcher_profiles (
            id INTEGER PRIMARY KEY,
            pitcher_id INTEGER NOT NULL REFERENCES players(player_id),
            season INTEGER NOT NULL,
            count_state TEXT,
            entropy REAL NOT NULL,
            total_pitches INTEGER NOT NULL,
            updated_at TEXT DEFAULT (datetime('now')),
            UNIQUE(pitcher_id, season, count_state)
        );

        CREATE TABLE IF NOT EXISTS games (
            game_pk INTEGER PRIMARY KEY,
            game_date TEXT NOT NULL,
            home_team_id INTEGER REFERENCES teams(team_id),
            away_team_id INTEGER REFERENCES teams(team_id),
            status TEXT,
            data_fetched INTEGER DEFAULT 0,
            created_at TEXT DEFAULT (datetime('now'))
        );
        ",
    )?;
    Ok(())
}

pub fn create_indexes(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        CREATE INDEX IF NOT EXISTS idx_plays_game ON plays(game_pk);
        CREATE INDEX IF NOT EXISTS idx_plays_batter ON plays(batter_id);
        CREATE INDEX IF NOT EXISTS idx_plays_pitcher ON plays(pitcher_id);
        CREATE INDEX IF NOT EXISTS idx_pitches_play ON pitches(play_id);
        CREATE INDEX IF NOT EXISTS idx_offense_trans_lookup ON offense_transitions(season, team_id, state_from);
        CREATE INDEX IF NOT EXISTS idx_pitch_trans_lookup ON pitch_transitions(pitcher_id, season, count_state);
        CREATE INDEX IF NOT EXISTS idx_pitcher_profiles_lookup ON pitcher_profiles(pitcher_id, season);
        CREATE INDEX IF NOT EXISTS idx_games_date ON games(game_date);
        CREATE INDEX IF NOT EXISTS idx_games_status ON games(status, data_fetched);
        ",
    )?;
    Ok(())
}
