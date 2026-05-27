<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getDbStatus, importSeason, onImportProgress } from "$lib/api";
  import type { DbStatus, ImportProgress, ImportResult } from "$lib/types";

  let status = $state<DbStatus | null>(null);
  let error = $state<string | null>(null);
  let importing = $state(false);
  let progress = $state<ImportProgress | null>(null);
  let result = $state<ImportResult | null>(null);
  let startTime = $state(0);
  let elapsed = $state("");

  let unlisten: (() => void) | null = null;
  let timer: ReturnType<typeof setInterval> | null = null;

  let isEmpty = $derived(status !== null && status.gamesCount === 0);
  let hasData = $derived(status !== null && status.gamesCount > 0);

  function fmt(n: number): string {
    return n.toLocaleString();
  }

  function fmtElapsed(ms: number): string {
    const s = Math.floor(ms / 1000);
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return m > 0 ? `${m}m ${sec}s` : `${sec}s`;
  }

  async function loadStatus() {
    try {
      status = await getDbStatus();
      error = null;
    } catch (e) {
      error = String(e);
    }
  }

  async function startImport() {
    importing = true;
    result = null;
    progress = null;
    startTime = Date.now();

    timer = setInterval(() => {
      elapsed = fmtElapsed(Date.now() - startTime);
    }, 1000);

    unlisten = await onImportProgress((p) => {
      progress = p;
    });

    try {
      result = await importSeason();
      await loadStatus();
    } catch (e) {
      error = String(e);
    } finally {
      importing = false;
      if (unlisten) {
        unlisten();
        unlisten = null;
      }
      if (timer) {
        clearInterval(timer);
        timer = null;
      }
      elapsed = fmtElapsed(Date.now() - startTime);
    }
  }

  onMount(loadStatus);

  onDestroy(() => {
    if (unlisten) unlisten();
    if (timer) clearInterval(timer);
  });
</script>

<h1 class="page-title">MLB Markov</h1>
<p class="page-sub">Markov chain models applied to real MLB play-by-play data. Every at-bat and every pitch from the current season, analyzed for patterns.</p>

<div class="grid">
  <a href="/offense" class="card link-card">
    <h3>Offense: What Happens Next?</h3>
    <p>
      Every at-bat moves the game between base/out situations. This model tracks
      every transition across the entire season and computes how many runs teams
      actually score from each situation. Compare any team to league average to
      find where they over- or underperform. Includes a momentum analysis — does
      scoring early in an inning lead to more scoring?
    </p>
    <span class="card-link">Open Offense →</span>
  </a>

  <a href="/pitching" class="card link-card">
    <h3>Pitching: What's Coming Next?</h3>
    <p>
      Look up any pitcher and see what they throw at every count. The model
      tracks pitch-to-pitch sequences — after a fastball at 0-2, what comes
      next? Select a count to see how a pitcher's approach changes when ahead,
      behind, or even. Includes a predictability score measuring how easy it is
      to guess the next pitch.
    </p>
    <span class="card-link">Open Pitching →</span>
  </a>
</div>

<div class="grid">
  <div class="card">
    <h3>Data</h3>

    {#if error}
      <p class="bad">{error}</p>
    {/if}

    {#if isEmpty && !importing && !result}
      <div class="empty-state">
        <p><strong>No data loaded yet.</strong> Click below to download play-by-play data for every completed game this season.</p>
        <p class="muted detail">
          First run fetches 800+ games, 60,000+ plays, 200,000+ pitches. Takes about <strong>3-4 minutes</strong>.
          After that, updates only pull new games (a few seconds).
        </p>
      </div>
    {/if}

    {#if hasData}
      <div class="stats-row">
        <div class="stat">
          <span class="stat-value mono">{fmt(status!.gamesCount)}</span>
          <span class="stat-label muted">Games</span>
        </div>
        <div class="stat">
          <span class="stat-value mono">{fmt(status!.playsCount)}</span>
          <span class="stat-label muted">Plays</span>
        </div>
        <div class="stat">
          <span class="stat-value mono">{fmt(status!.pitchesCount)}</span>
          <span class="stat-label muted">Pitches</span>
        </div>
        <div class="stat">
          <span class="stat-value mono">{fmt(status!.playersCount)}</span>
          <span class="stat-label muted">Players</span>
        </div>
      </div>

      {#if status!.lastGameDate}
        <p class="last-update muted">
          Data through <strong>{status!.lastGameDate}</strong>
          {#if status!.pendingGames > 0}
            · at least <strong>{status!.pendingGames}</strong> new games available
          {/if}
        </p>
      {/if}
    {/if}

  <div class="import-section">
    {#if importing}
      <div class="progress-area">
        {#if progress && progress.total > 0}
          <div class="progress-bar">
            <div
              class="progress-fill"
              style="width: {(progress.current / progress.total) * 100}%"
            ></div>
          </div>
          <p class="mono progress-text">{progress.message}</p>
        {:else if progress}
          <p class="mono progress-text">{progress.message}</p>
        {:else}
          <p class="muted">Starting import...</p>
        {/if}
        {#if elapsed}
          <p class="muted elapsed">{elapsed}</p>
        {/if}
      </div>
    {:else}
      <button class="accent" onclick={startImport} disabled={importing}>
        {isEmpty ? "Bootstrap Current Season" : "Update Data"}
      </button>
      {#if hasData}
        <span class="muted update-hint">Pulls only new games since last update.</span>
      {/if}
    {/if}

    {#if result && !importing}
      <div class="result-summary">
        <p>
          Imported <strong>{fmt(result.gamesImported)}</strong> games,
          <strong>{fmt(result.playsInserted)}</strong> plays,
          <strong>{fmt(result.pitchesInserted)}</strong> pitches
          in {elapsed}.
          {#if result.gamesSkipped > 0}
            <span class="muted">{result.gamesSkipped} games skipped — API returned errors or data wasn't available. They'll retry on the next update.</span>
          {/if}
        </p>
      </div>
    {/if}
  </div>
  </div>

  <div class="card use-card">
    <h3>What Can You Do With This?</h3>
    <ul>
      <li><strong>Trace an inning:</strong> Hover the offense heatmap to follow the chain — see the probability of each transition and how expected runs change with every at-bat.</li>
      <li><strong>Compare teams:</strong> Select a team and see where they score more or fewer runs than league average from each base/out situation.</li>
      <li><strong>Test momentum:</strong> Does scoring early in an inning lead to more scoring? The momentum tab splits cold vs hot innings to find out.</li>
      <li><strong>Scout a pitcher:</strong> Look up any pitcher, pick a count, and see what they throw after each pitch type. Find where they become predictable.</li>
    </ul>
  </div>
</div>

<style>
  .page-title {
    font-size: 1.5rem;
    margin-bottom: 2px;
  }
  .page-sub {
    font-size: 0.88rem;
    margin: 0 0 16px;
    max-width: 700px;
  }
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-bottom: 16px;
  }
  .link-card {
    text-decoration: none;
    color: inherit;
    transition: border-color 150ms ease;
  }
  .link-card:hover {
    border-color: var(--accent);
    text-decoration: none;
  }
  .link-card p {
    font-size: 0.84rem;
    line-height: 1.45;
  }
  .card-link {
    font-size: 0.84rem;
    color: var(--accent);
    font-weight: 600;
  }
  .use-card ul {
    margin: 8px 0 0;
    padding-left: 1.2em;
    color: var(--ink-soft);
  }
  .use-card li {
    font-size: 0.84rem;
    line-height: 1.45;
    margin-bottom: 8px;
  }
  .empty-state p {
    margin: 0 0 8px;
    font-size: 0.9rem;
  }
  .detail {
    font-size: 0.82rem;
    line-height: 1.45;
  }
  .stats-row {
    display: flex;
    gap: 32px;
    margin-top: 8px;
  }
  .stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .stat-value {
    font-size: 1.4rem;
    font-weight: 600;
  }
  .stat-label {
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .last-update {
    font-size: 0.8rem;
    margin: 10px 0 0;
  }
  .bad {
    color: var(--bad);
  }
  .import-section {
    margin-top: 14px;
    padding-top: 12px;
    border-top: 1px solid var(--line-soft);
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }
  .update-hint {
    font-size: 0.78rem;
  }
  .progress-area {
    display: flex;
    flex-direction: column;
    gap: 6px;
    width: 100%;
  }
  .progress-bar {
    height: 8px;
    background: var(--bg-soft);
    border-radius: 4px;
    overflow: hidden;
  }
  .progress-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 4px;
    transition: width 200ms ease;
  }
  .progress-text {
    font-size: 0.82rem;
    margin: 0;
  }
  .elapsed {
    font-size: 0.75rem;
    margin: 0;
  }
  .result-summary {
    width: 100%;
  }
  .result-summary p {
    margin: 0;
    font-size: 0.88rem;
  }
</style>
