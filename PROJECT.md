# MLB Markov

State transition models for MLB. Two views -- offense and pitching -- both built on Markov chain logic.

## Concept

**Offense: At-Bat State Transitions**
- 25 base-out states: 24 active (0/1/2 outs x 8 base configurations: empty, 1B, 2B, 3B, 1B+2B, 1B+3B, 2B+3B, loaded) + 1 absorbing state (3 outs, end of inning)
- Model probability of transitioning between states on each plate appearance
- Calculate expected runs from any state via the fundamental matrix (requires the absorbing state)
- Heatmap visualization -- which states produce runs, which are dead ends
- Compare team actual transition rates vs league average

**Pitching: Pitch Sequence Predictability**
- Model what pitch type follows what pitch type, broken down by count
- Transition matrix per pitcher: FF, FS, FC, etc.
- Identify most predictable pitchers (low entropy) vs most unpredictable (high entropy)
- Break down by situation: ahead in count, behind, even, two-strike
- Note: the Markov (memoryless) assumption is weaker for pitch sequences -- a pitcher's next pitch often depends on the last 2-3 pitches, not just the most recent. First-order chains are a useful simplification but this is a known limitation. Consider 2nd-order chains as a future enhancement.

## Data Source

### MLB Stats API (Primary)
- Free, no key required
- Base endpoint: `statsapi.mlb.com/api/v1/` (some detailed endpoints use `v1.1` -- verify per route)
- **Pitch-by-pitch data**: `game/{gamePk}/playByPlay` -- every pitch, type, result, count, runners
- **Schedule**: `schedule` -- game IDs by date range
- **Pitch arsenals**: `people/{playerId}` + stats endpoints for pitch mix percentages
- This gives us everything. Pitch types, velocities, sequences, base runners, outs, outcomes.

### What We Need From the API
1. **Play-by-play** for a large sample of games (full season or two)
   - Each play: pitch type, pitch result, count before/after, base state before/after, outs before/after, runs scored
2. **Roster data** for pitcher lookup (who threw what)
3. **Schedule** to iterate games by date

### Data Strategy
- Fetch and cache locally (SQLite). Don't re-fetch.
- One-time bootstrap: pull a full season of play-by-play data
- Incremental updates: pull new games as they complete
- Store raw plays + derived transition matrices
- **Recomputation**: on incremental imports, recompute transition matrices for affected teams/pitchers only. Full recompute available as a manual action for rebuilds or schema changes.

### Data Quality / Edge Cases
- **Suspended/postponed games**: skip games without status `Final`. Track status in `games` table, re-check on next import.
- **Incomplete play data**: some plays may lack pitch-level detail (e.g., intentional walks pre-2017). Record the plate appearance outcome for offense transitions but skip pitch sequence tracking.
- **Intentional walks / HBP / errors**: these are valid state transitions for offense (they change base-out state). Map them like any other event.
- **Pinch runners**: a runner substitution doesn't change the base-out state, only who occupies it. The Markov model doesn't track runner identity, so no special handling needed.

## Tech Stack

### Core
- **Tauri v2** -- desktop shell, native macOS
- **Rust (backend)** -- data processing, transition matrix math, SQLite queries
- **SvelteKit (frontend)** -- UI, charts, interactions. Uses `adapter-static` for Tauri compatibility (no SSR).
- **TypeScript** -- frontend logic

### Charts & Visualization
- **D3.js** -- heatmap for state transition matrices, flexible enough for custom layouts
- SVG-based for the base-out state diagram (interactive, clickable states)
- Color-coded transition probabilities (green = high probability, red = low)

### Data
- **SQLite** via Rusqlite -- local cache, fast queries
- Store: plays, transition matrices, pitcher profiles
- Precompute transition matrices on import, store results

### Build
- Standard Tauri v2 + SvelteKit (adapter-static) setup (same as mlb-pe, oddsdesk, futures-journal)
- GitHub Actions CI with multi-platform releases

## Architecture

```
mlb-markov/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── db/              # SQLite setup, migrations
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs
│   │   │   └── migrations/
│   │   ├── api/             # MLB Stats API client
│   │   │   ├── mod.rs
│   │   │   ├── client.rs    # HTTP client, rate limiting
│   │   │   ├── schedule.rs  # Game schedule fetcher
│   │   │   └── plays.rs     # Play-by-play fetcher + parser
│   │   ├── markov/          # Core logic
│   │   │   ├── mod.rs
│   │   │   ├── states.rs    # Base-out state definitions (24 active + 1 absorbing)
│   │   │   ├── transitions.rs # Transition matrix builder
│   │   │   ├── expected_runs.rs # Run expectancy via fundamental matrix
│   │   │   ├── pitch_seq.rs # Pitch sequence Markov chains
│   │   │   └── entropy.rs   # Predictability / entropy calculations
│   │   └── commands/        # Tauri IPC commands
│   │       ├── mod.rs
│   │       ├── data.rs      # Import/update game data
│   │       ├── offense.rs   # At-bat state queries
│   │       └── pitching.rs  # Pitch sequence queries
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                     # SvelteKit frontend
│   ├── app.html
│   ├── lib/
│   │   ├── components/
│   │   │   ├── StateHeatmap.svelte    # 25-state transition heatmap
│   │   │   ├── DiamondDiagram.svelte  # Interactive base-out state visual
│   │   │   ├── PitchMatrix.svelte     # Pitcher sequence transition grid
│   │   │   ├── PitcherCard.svelte     # Individual pitcher profile
│   │   │   └── TeamSelector.svelte
│   │   ├── charts/
│   │   │   └── heatmap.ts             # D3.js heatmap rendering
│   │   └── stores/
│   │       ├── data.ts
│   │       └── ui.ts
│   └── routes/
│       ├── +layout.svelte
│       ├── +page.svelte               # Dashboard / home
│       ├── offense/+page.svelte       # At-bat transitions view
│       └── pitching/+page.svelte      # Pitch sequences view
├── static/
├── package.json
├── svelte.config.js
├── vite.config.ts
└── tsconfig.json
```

## Data Model (SQLite)

### Tables

```sql
-- Lookup: teams
CREATE TABLE teams (
    team_id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    abbreviation TEXT NOT NULL,
    league TEXT,
    division TEXT,
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Lookup: players
CREATE TABLE players (
    player_id INTEGER PRIMARY KEY,
    full_name TEXT NOT NULL,
    primary_position TEXT,
    throws TEXT, -- 'L' or 'R'
    bats TEXT,   -- 'L', 'R', or 'S'
    team_id INTEGER REFERENCES teams(team_id),
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Raw play-by-play data
CREATE TABLE plays (
    id INTEGER PRIMARY KEY,
    game_pk INTEGER NOT NULL,
    inning INTEGER NOT NULL,
    half TEXT NOT NULL,          -- 'top' or 'bottom'
    event TEXT NOT NULL,         -- 'Single', 'Walk', 'Strikeout', etc
    outs_before INTEGER,
    outs_after INTEGER,
    bases_before TEXT,           -- 3-char string: '000' = empty, '110' = 1B+2B, '111' = loaded
    bases_after TEXT,
    runs_scored INTEGER DEFAULT 0,
    batter_id INTEGER REFERENCES players(player_id),
    pitcher_id INTEGER REFERENCES players(player_id),
    created_at TEXT DEFAULT (datetime('now'))
);

-- Individual pitches within plays
CREATE TABLE pitches (
    id INTEGER PRIMARY KEY,
    play_id INTEGER REFERENCES plays(id),
    pitch_number INTEGER,
    pitch_type TEXT,             -- 'FF', 'SL', 'CH', 'CU', 'SI', 'FC', etc
    pitch_type_desc TEXT,        -- '4-Seam Fastball', 'Slider', etc
    release_speed REAL,
    count_balls INTEGER,
    count_strikes INTEGER,
    result TEXT,                 -- 'Ball', 'Strike', 'Foul', 'In play', etc
    created_at TEXT DEFAULT (datetime('now'))
);

-- Precomputed at-bat transition matrices
-- state format: 'outs_bases' e.g. '0_000' = 0 outs, bases empty; '3_---' = absorbing (end of inning)
CREATE TABLE offense_transitions (
    id INTEGER PRIMARY KEY,
    season INTEGER NOT NULL,
    team_id INTEGER REFERENCES teams(team_id), -- NULL = league average
    state_from TEXT NOT NULL,
    state_to TEXT NOT NULL,
    occurrences INTEGER NOT NULL,
    probability REAL NOT NULL,
    avg_runs_scored REAL DEFAULT 0,
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Precomputed pitch sequence matrices
CREATE TABLE pitch_transitions (
    id INTEGER PRIMARY KEY,
    pitcher_id INTEGER NOT NULL REFERENCES players(player_id),
    season INTEGER NOT NULL,
    count_state TEXT,            -- '0-0', '1-0', '0-1', etc (NULL = all counts)
    pitch_from TEXT NOT NULL,    -- 'FF', 'SL', etc
    pitch_to TEXT NOT NULL,
    occurrences INTEGER NOT NULL,
    probability REAL NOT NULL,
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Pitcher-level aggregates (entropy lives here, not per-transition)
CREATE TABLE pitcher_profiles (
    id INTEGER PRIMARY KEY,
    pitcher_id INTEGER NOT NULL REFERENCES players(player_id),
    season INTEGER NOT NULL,
    count_state TEXT,            -- NULL = overall, '0-0', '1-0', etc for per-count
    entropy REAL NOT NULL,
    total_pitches INTEGER NOT NULL,
    updated_at TEXT DEFAULT (datetime('now')),
    UNIQUE(pitcher_id, season, count_state)
);

-- Game cache (avoid re-fetching)
CREATE TABLE games (
    game_pk INTEGER PRIMARY KEY,
    game_date TEXT NOT NULL,
    home_team_id INTEGER REFERENCES teams(team_id),
    away_team_id INTEGER REFERENCES teams(team_id),
    status TEXT,
    data_fetched INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now'))
);
```

### Indexes

```sql
CREATE INDEX idx_plays_game ON plays(game_pk);
CREATE INDEX idx_plays_batter ON plays(batter_id);
CREATE INDEX idx_plays_pitcher ON plays(pitcher_id);
CREATE INDEX idx_pitches_play ON pitches(play_id);
CREATE INDEX idx_offense_trans_lookup ON offense_transitions(season, team_id, state_from);
CREATE INDEX idx_pitch_trans_lookup ON pitch_transitions(pitcher_id, season, count_state);
CREATE INDEX idx_pitcher_profiles_lookup ON pitcher_profiles(pitcher_id, season);
CREATE INDEX idx_games_date ON games(game_date);
CREATE INDEX idx_games_status ON games(status, data_fetched);
```

## Learning Tab: The Math Behind It

A built-in education page that explains the concepts so anyone can understand what they're looking at.

### Sections

1. **What is a Markov Chain?**
   - Plain English: a system where the next state depends only on where you are right now, not how you got there
   - The "no memory" property -- why it fits baseball (the runner on 2nd doesn't care how he got there)
   - Simple visual: state diagram with arrows and probabilities

2. **The 25 Base-Out States**
   - Why 25? (3 out levels x 8 base configurations = 24 active states + 1 absorbing state for 3 outs / end of inning)
   - Interactive diagram -- click any state, see its transition probabilities
   - Why this framework matters (every plate appearance is a state transition)
   - The absorbing state: once 3 outs are recorded, the inning is over -- no transitions out

3. **Expected Runs (RE24)**
   - How to calculate: solve the fundamental matrix (I - Q)^-1 where Q is the transient portion of the transition matrix
   - Why some states are worth more than you'd think (man on 2nd, 0 outs vs man on 3rd, 2 outs)
   - The full RE24 table -- the "money chart" of baseball analytics

4. **Transition Matrices Explained**
   - What a matrix is (grid of probabilities)
   - How to read one: row = current state, column = next state, cell = probability
   - Real example from the data: "from 'man on 1st, 0 outs', here's where you end up"

5. **Pitch Sequence Entropy (Predictability)**
   - Shannon Entropy: a measure of how "surprising" a pitcher's next pitch is
   - Low entropy = predictable (he always throws fastball when behind)
   - High entropy = unpredictable (equal mix, hitter can't sit on anything)
   - Formula shown with a real example ("Pitcher X: FF 60%, SL 25%, CH 15% → entropy = Y")
   - Why it matters: predictable pitchers get hit harder

### Design
- Svelte component-based, one section per route
- Math rendered with KaTeX or similar (clean formulas, not images)
- Each concept tied back to a real data point from the app ("here's what this looks like for the 2024 Dodgers")
- Collapsible sections, progress through top to bottom

## Views / Features

### Offense Tab: At-Bat State Transitions
1. **League Average Heatmap** -- 25x25 grid, color-coded transition probabilities (absorbing state row is all zeros)
2. **Team Comparison** -- select a team, see how their transitions deviate from league average
3. **Expected Runs Table** -- 25 states, expected runs from each (the classic RE24 table; absorbing state = 0)
4. **Interactive Diamond** -- click a base-out state, see where you're most likely to end up and what it's worth
5. **Season Slider** -- compare across seasons (are teams getting more efficient?)

### Pitching Tab: Pitch Sequence Predictability
1. **Pitcher Search** -- look up any pitcher, see their transition matrix
2. **Predictability Score** -- Shannon entropy from `pitcher_profiles`. Low = predictable, high = unpredictable
3. **Count Breakdown** -- see how approach changes by count (ahead, behind, even)
4. **League Leaderboard** -- most/least predictable pitchers in MLB
5. **Pitch Mix Pie Chart** -- what they throw, how often

## API Rate Limiting
- MLB Stats API is generous but not unlimited
- Cache everything locally, only fetch what we don't have
- Batch game imports (pull a week at a time during bootstrap)
- Respect 1-second delay between requests during bulk imports

## Scope for V1 Release
Keep it tight for the first version:
- Bootstrap: pull 2024 season play-by-play
- Offense view: heatmap + expected runs table
- Pitching view: one pitcher's sequence matrix + predictability score
- Post format: "I built a Markov chain model for MLB at-bats. Here's what every base-out state is worth."

## Later Enhancements
- Live game integration (real-time win probability as states change)
- Multi-season trends (is the game getting more predictable?)
- Batter-specific transitions (not just team)
- Export matrices as CSV for bettors/modelers
- 2nd-order Markov chains for pitch sequences (pitch N depends on pitches N-1 and N-2)
- Interactive Sandbox in Learning Tab: simplified 3-state Markov chain where users adjust probabilities and watch the system evolve ("What happens if I increase walk rate by 5%?")
