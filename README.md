# MLB Markov

A desktop app that applies Markov chain models to MLB play-by-play data. Two analysis views: **offense** (base-out state transitions and run expectancy) and **pitching** (pitch sequence predictability via Shannon entropy).

Built with [Tauri v2](https://v2.tauri.app) (Rust backend + SvelteKit frontend). Data from the free [MLB Stats API](https://statsapi.mlb.com).

## What It Does

**Offense: At-Bat Transitions**
- Models every plate appearance as a transition between 25 base-out states (24 active + 1 absorbing)
- Computes the full transition probability matrix from a season of play-by-play data
- Calculates expected runs from any state (RE24) via the fundamental matrix
- Compare any team against league average with the heatmap visualization

**Pitching: Sequence Predictability**
- Builds a pitch-type Markov chain for any pitcher (what pitch follows what pitch)
- Computes Shannon entropy as a predictability score — low entropy = predictable, high = unpredictable
- Breaks down entropy by count to show how approach changes when ahead vs behind

**Learning Tab**
- Built-in educational content explaining Markov chains, the 25-state model, RE24 math, transition matrices, and entropy
- Rendered formulas via KaTeX
- Explains the data pipeline so you understand what you're downloading

## Data Pipeline

When you click **"Bootstrap Current Season"**, the app fetches play-by-play data for every completed game in the current MLB season from the Stats API. This includes every plate appearance and every individual pitch.

**Initial load:** ~800-1,200 games mid-season, each requiring an API call. Takes about 3-4 minutes with the 250ms rate limit. This is a one-time cost.

**Incremental updates:** After the initial bootstrap, only new games (completed since your last import) are fetched. During the season, that's ~12-15 games per day — a few seconds.

**Scale:** A full season is roughly 150,000+ plays and 700,000+ pitches, stored locally in SQLite.

## Development

### Prerequisites
- [Rust](https://rustup.rs) (stable)
- [Node.js](https://nodejs.org) (18+)
- [pnpm](https://pnpm.io) (10+)

### Setup
```bash
pnpm install
pnpm tauri dev
```

### Build
```bash
pnpm tauri build
```

The production binary lands in `src-tauri/target/release/bundle/`.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri v2 |
| Backend | Rust (reqwest, rusqlite, serde, tokio) |
| Frontend | SvelteKit (adapter-static, Svelte 5) |
| Charts | D3.js (SVG heatmaps) |
| Math rendering | KaTeX |
| Database | SQLite (WAL mode, bundled via rusqlite) |
| Data source | MLB Stats API (free, no key) |

## Architecture

```
src-tauri/src/
├── api/           MLB Stats API client (schedule, play-by-play parser)
├── commands/      Tauri IPC commands (data, offense, pitching)
├── db/            SQLite setup, schema, migrations
├── markov/        Core math (states, transitions, expected runs, entropy)
└── lib.rs         AppState, setup, season detection

src/
├── lib/
│   ├── charts/    D3.js heatmap renderer
│   ├── components/  Svelte 5 components (heatmap, tables, pitcher card, etc.)
│   ├── api.ts     Tauri invoke wrappers
│   └── types.ts   TypeScript types matching Rust serialization
└── routes/
    ├── +page.svelte          Home (DB status, import)
    ├── offense/+page.svelte  Transition heatmap + RE24
    ├── pitching/+page.svelte Pitcher search + entropy
    └── learning/+page.svelte Educational content
```

## License

MIT
