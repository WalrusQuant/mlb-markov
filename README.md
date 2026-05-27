# MLB Markov

A desktop app that applies Markov chain models to real MLB play-by-play data. Built with [Tauri v2](https://v2.tauri.app) (Rust + SvelteKit). Data from the free [MLB Stats API](https://statsapi.mlb.com).

## What It Does

### Offense: What Happens Next?
Every at-bat moves the game between base/out situations. This model tracks every transition and computes how many runs teams actually score from each situation.

- **Team vs League Average** — select any team and see where they score more or fewer runs than average, sorted by biggest difference
- **Momentum Analysis** — do teams score more from the same situation when runs have already scored in the inning? The data answers it
- **Trace the chain** — hover any cell in the heatmap to see the transition probability and how expected runs change from one state to the next

### Pitching: What's Coming Next?
Look up any pitcher and see what they throw at every count.

- **Count-specific sequences** — select a count (0-2, 3-1, etc.) to see the pitch-to-pitch transition matrix for that count. After a fastball at 0-2, what comes next?
- **Predictability score** — measures how easy it is to guess the next pitch. Updates per count so you can see where a pitcher becomes predictable
- **Full pitch type coverage** — tracks every pitch type in the pitcher's arsenal across the entire season

### Learning Tab
Built-in educational content explaining Markov chains, the 25 base-out states, run expectancy math, transition matrices, Shannon entropy, and the data pipeline. Formulas rendered with KaTeX.

## Data Pipeline

On first launch, click **"Bootstrap Current Season"** to download play-by-play data for every completed game this season.

- **Initial load:** 800+ games, 60,000+ plays, 200,000+ pitches. Takes about 3-4 minutes.
- **Updates:** After the first load, click "Update Data" to pull only new games since your last import. Typically 12-15 games per day — a few seconds.
- **All local:** Everything is stored in a SQLite database on your machine. No account, no cloud, no API key.

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
| Charts | D3.js (SVG heatmaps with interactive tooltips) |
| Math rendering | KaTeX |
| Database | SQLite (WAL mode, bundled via rusqlite) |
| Data source | MLB Stats API (free, no key) |

## Architecture

```
src-tauri/src/
├── api/           MLB Stats API client (schedule, play-by-play parser)
├── commands/      Tauri IPC commands (data, offense, pitching)
├── db/            SQLite setup, schema, migrations
├── markov/        Core math (states, transitions, expected runs, momentum, entropy)
└── lib.rs         AppState, setup, season detection

src/
├── lib/
│   ├── charts/    D3.js heatmap renderer with tooltips
│   ├── components/  Svelte 5 components
│   │   ├── StateHeatmap.svelte       25-state offense heatmap
│   │   ├── RunComparisonTable.svelte  Team vs league comparison
│   │   ├── MomentumTable.svelte       Cold vs hot innings
│   │   ├── InsightCallouts.svelte     Highlight biggest differences
│   │   ├── PitchMatrix.svelte         Pitch-type transition heatmap
│   │   ├── ExpectedRunsTable.svelte   RE24 table
│   │   └── TeamSelector.svelte        Team dropdown
│   ├── api.ts     Tauri invoke wrappers
│   └── types.ts   TypeScript types matching Rust serialization
└── routes/
    ├── +page.svelte          Home (data status, import, use cases)
    ├── offense/+page.svelte  Team edge + momentum analysis
    ├── pitching/+page.svelte Count-specific pitch sequencing
    └── learning/+page.svelte Educational content
```

## License

MIT
