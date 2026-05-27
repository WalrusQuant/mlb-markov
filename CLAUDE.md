# CLAUDE.md

## Project Overview
MLB Markov — Tauri v2 desktop app applying Markov chain models to MLB play-by-play data. Two analysis views (offense state transitions, pitching entropy) plus a learning/educational tab.

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
- Use `$state`, `$derived`, `$effect`, `$props()` — NOT stores or Svelte 4 syntax
- Event handling: callback props pattern (`onchange?: () => void` in `$props()`), NOT `on:change` or `createEventDispatcher`
- Layout uses `{@render children?.()}`

### CSS
- CSS variables design system in `src/app.css` — light/dark mode via `prefers-color-scheme`
- Use `.card`, `.badge`, `.muted`, `.shell`, `.formula` classes
- No Tailwind

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

### Markov Engine
- 25 states: 24 active + 1 absorbing ("3_---") in `src-tauri/src/markov/states.rs`
- RE24 via fundamental matrix N = (I-Q)^-1, Gaussian elimination with partial pivoting
- Shannon entropy: `-Σ(p·ln(p))`, weighted average for matrix entropy

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
  lib/components/     Svelte 5 components
  routes/             Home, offense, pitching, learning
```

## Common Pitfalls
- rusqlite `Mutex<Connection>` must be unlocked before awaiting — scope lock guards in blocks
- MLB API `runners` array only includes runners who moved — must track base state sequentially
- Svelte 5 uses callback props, not `on:event` directives
- KaTeX CSS loaded via CDN link in `Tex.svelte` head
- Delete ALL `mlbmarkov.db*` files (including .db-shm, .db-wal) when resetting the database
