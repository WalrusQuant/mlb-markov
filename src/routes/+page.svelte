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
      result = await importSeason(2024);
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

<h1>MLB Markov</h1>
<p>State transition models for MLB offense and pitching, built on Markov chain logic.</p>

<div class="grid">
  <div class="card">
    <h3>Offense: At-Bat Transitions</h3>
    <p>25 base-out state transition heatmap and expected runs (RE24) computed from play-by-play data.</p>
    <a href="/offense">View Offense</a>
  </div>

  <div class="card">
    <h3>Pitching: Sequence Predictability</h3>
    <p>Pitch-type transition matrices and Shannon entropy scores. Find the most predictable pitchers in MLB.</p>
    <a href="/pitching">View Pitching</a>
  </div>
</div>

<hr />

<div class="card">
  <h3>Database Status</h3>
  {#if error}
    <p class="bad">{error}</p>
  {:else if !status}
    <p class="muted">Loading...</p>
  {:else if status.gamesCount === 0 && !importing && !result}
    <p class="muted">Database empty -- import data to get started.</p>
  {/if}

  <div class="stats-row">
    <div class="stat">
      <span class="stat-value mono">{status ? fmt(status.gamesCount) : "--"}</span>
      <span class="stat-label muted">Games</span>
    </div>
    <div class="stat">
      <span class="stat-value mono">{status ? fmt(status.playsCount) : "--"}</span>
      <span class="stat-label muted">Plays</span>
    </div>
    <div class="stat">
      <span class="stat-value mono">{status ? fmt(status.pitchesCount) : "--"}</span>
      <span class="stat-label muted">Pitches</span>
    </div>
    <div class="stat">
      <span class="stat-value mono">{status ? fmt(status.playersCount) : "--"}</span>
      <span class="stat-label muted">Players</span>
    </div>
  </div>

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
        Bootstrap 2024 Season
      </button>
    {/if}

    {#if result && !importing}
      <div class="result-summary">
        <p>
          Imported <strong>{fmt(result.gamesImported)}</strong> games,
          <strong>{fmt(result.playsInserted)}</strong> plays,
          <strong>{fmt(result.pitchesInserted)}</strong> pitches
          {#if result.gamesSkipped > 0}
            ({result.gamesSkipped} skipped)
          {/if}
          in {elapsed}.
        </p>
      </div>
    {/if}
  </div>
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-bottom: 24px;
  }
  .stats-row {
    display: flex;
    gap: 32px;
    margin-top: 12px;
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
  .bad {
    color: var(--bad);
  }
  .import-section {
    margin-top: 20px;
    padding-top: 16px;
    border-top: 1px solid var(--line-soft);
  }
  .progress-area {
    display: flex;
    flex-direction: column;
    gap: 8px;
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
    font-size: 0.85rem;
    margin: 0;
  }
  .elapsed {
    font-size: 0.78rem;
    margin: 0;
  }
  .result-summary {
    margin-top: 12px;
  }
  .result-summary p {
    margin: 0;
  }
</style>
