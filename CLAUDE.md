# CLAUDE.md

## Project Overview
MLB Markov — Tauri v2 desktop app applying Markov chain models to MLB play-by-play data. Offense view (base-out state transitions, team vs league comparison, momentum analysis), pitching view (count-specific pitch sequencing and predictability), and a learning/educational tab.

## Tech Stack
- **Backend:** Rust, Tauri v2, rusqlite (bundled, WAL mode), reqwest (rustls-tls), serde, tokio, chrono, anyhow, thiserror 2
- **Frontend:** SvelteKit with adapter-static (SSR disabled), Svelte 5, TypeScript, D3.js, KaTeX
- **Database:** SQLite — 7 tables (teams, players, plays, pitches, games, offense_transitions, pitch_transitions, pitcher_profiles)
- **Data source:** MLB Stats API (free, no key) — statsapi.mlb.com/api/v1/

## Key Patterns

### Tauri IPC
- Rust commands in `src-tauri/src/commands/` use `#[tauri::command]` with `serde(rename_all = "camelCase")`
- Frontend wrappers in `src/lib/api.ts` use `invoke()` from `@tauri-apps/api/core`
- `AppState { db: Mutex<Connection>, db_path: String }` shared via Tauri state
- Season determined server-side via `default_season()` using `Utc::now()` — all command season params are `Option<i32>`

### Svelte 5 Reactivity
- Use `$state`, `$derived`, `$derived.by()`, `$effect`, `$props()` — NOT stores or Svelte 4 syntax
- Event handling: callback props pattern (`onchange?: () => void` in `$props()`), NOT `on:change` or `createEventDispatcher`
- Layout uses `{@render children?.()}`

### CSS
- CSS variables design system in `src/app.css` — light/dark mode via `prefers-color-scheme`
- Use `.card`, `.badge`, `.muted`, `.shell`, `.formula` classes
- No Tailwind

### Viewport Layout
- Offense and Pitching pages are viewport-locked: `overflow: hidden`, `height: calc(100vh - Xpx)`
- Home and Learning pages scroll normally via `overflow: auto` on `.main`
- `overscroll-behavior: none` on html/body to prevent macOS elastic bounce
- Tauri window: 1200x800 default. All data pages must fit with zero scrolling.

### Database
- rusqlite directly (NOT tauri-plugin-sql)
- WAL mode, foreign keys ON
- Version-based migrations in `src-tauri/src/db/migrations.rs`
- Schema in `src-tauri/src/db/schema.rs`

### MLB API Parsing
- Play-by-play parser in `src-tauri/src/api/plays.rs` — most complex file
- Base state tracked as `[bool; 3]` across plays within half-inning (runners array only includes runners who MOVED)
- `outs_before = count.outs - number_of_runners_with_isOut_true`
- Liberal `#[serde(default)]` on all API DTOs
- `isOut` field can be `null` in API — uses custom `deserialize_null_bool` to handle

### Markov Engine
- 25 states: 24 active + 1 absorbing ("3_---") in `src-tauri/src/markov/states.rs`
- RE24 via fundamental matrix N = (I-Q)^-1, Gaussian elimination with partial pivoting
- Shannon entropy: `-Σ(p·ln(p))`, weighted average for matrix entropy
- Momentum analysis: SQL window function splits plays into cold (0 runs scored in inning) vs hot (1+ runs scored), computes separate transition matrices

### D3 Heatmap
- `src/lib/charts/heatmap.ts` renders SVG with tooltip div (not SVG `<title>`)
- Tooltip shows friendly state names, probability, and expected runs at both states
- Tooltip flips left near right edge to prevent clipping
- Label sizes scale with matrix size: 14px for small (pitch types), 9px for medium, 7px for 25-state offense

## Commands
```bash
pnpm install          # install deps
pnpm tauri dev        # dev mode (frontend port 1420)
pnpm tauri build      # production build
pnpm check            # svelte-check type checking
```

## File Layout
```
src-tauri/src/
  lib.rs              AppState, setup, default_season(), command registry
  api/                MLB Stats API client + parsers
  commands/           Tauri IPC commands (data, offense, pitching)
  db/                 SQLite init, schema, migrations
  markov/             States, transitions, expected_runs, pitch_seq, entropy

src/
  lib/api.ts          All invoke() wrappers
  lib/types.ts        TypeScript types matching Rust camelCase serialization
  lib/charts/         D3.js heatmap renderer
  lib/components/     Svelte 5 components (heatmap, tables, comparison, momentum, pitcher)
  routes/             Home, offense, pitching, learning
```

## Common Pitfalls
- rusqlite `Mutex<Connection>` must be unlocked before awaiting — scope lock guards in blocks
- MLB API `runners` array only includes runners who moved — must track base state sequentially
- MLB API `isOut` can be `null` — use custom deserializer, not `#[serde(default)]` alone
- Svelte 5 uses callback props, not `on:event` directives
- KaTeX CSS loaded via CDN link in `app.html`
- Delete ALL `mlbmarkov.db*` files (including .db-shm, .db-wal) when resetting the database
- Offense/Pitching pages must fit 1200x800 viewport with zero scrolling — use flex layouts with `overflow: hidden`
