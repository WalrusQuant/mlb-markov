<script lang="ts">
  import { searchPitchers, getPitchSequences } from "$lib/api";
  import type { PitcherSearchResult, PitchSequenceBundle } from "$lib/types";
  import PitcherCard from "$lib/components/PitcherCard.svelte";
  import PitchMatrix from "$lib/components/PitchMatrix.svelte";
  import CountBreakdown from "$lib/components/CountBreakdown.svelte";

  let query = $state("");
  let results = $state<PitcherSearchResult[]>([]);
  let bundle = $state<PitchSequenceBundle | null>(null);
  let loading = $state(false);
  let searching = $state(false);
  let error = $state<string | null>(null);
  let season: number | undefined = undefined;

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  function onInput() {
    if (debounceTimer) clearTimeout(debounceTimer);
    if (query.length < 2) {
      results = [];
      return;
    }
    debounceTimer = setTimeout(async () => {
      searching = true;
      try {
        results = await searchPitchers(query, season);
      } catch (_) {
        results = [];
      } finally {
        searching = false;
      }
    }, 300);
  }

  async function selectPitcher(pitcher: PitcherSearchResult) {
    query = pitcher.fullName;
    results = [];
    loading = true;
    error = null;
    try {
      bundle = await getPitchSequences(pitcher.playerId, season);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
</script>

<div class="page-header">
  <div>
    <h1>Pitching: Sequence Predictability</h1>
    <p>Search for a pitcher to view their pitch-type transition matrix and predictability score.</p>
  </div>
  <div class="search-wrap">
    <input
      type="text"
      placeholder="Search pitcher name..."
      bind:value={query}
      oninput={onInput}
    />
    {#if results.length > 0}
      <div class="dropdown">
        {#each results as r}
          <button class="dropdown-item" onclick={() => selectPitcher(r)}>
            <span class="name">{r.fullName}</span>
            <span class="meta muted">{r.team} · {r.totalPitches.toLocaleString()} pitches</span>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>

{#if error}
  <div class="card">
    <p class="bad">{error}</p>
  </div>
{:else if loading}
  <div class="card">
    <p class="muted">Computing pitch sequence transitions...</p>
  </div>
{:else if bundle}
  <PitcherCard
    name={bundle.pitcherName}
    entropy={bundle.overallEntropy}
    totalPitches={bundle.totalPitches}
    pitchTypes={bundle.pitchTypes}
  />

  <div class="content-row">
    <div class="matrix-col">
      <PitchMatrix data={bundle.overallMatrix} title="Overall Pitch Transition Matrix" />
    </div>
    {#if bundle.countEntropy.length > 0}
      <div class="entropy-col">
        <CountBreakdown countEntropy={bundle.countEntropy} byCount={bundle.byCount} />
      </div>
    {/if}
  </div>
{:else}
  <div class="card">
    <p class="muted">Import play-by-play data from the Home page, then search for a pitcher above.</p>
  </div>
{/if}

<style>
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 24px;
    margin-bottom: 12px;
  }
  .page-header h1 {
    font-size: 1.5rem;
    margin-bottom: 2px;
  }
  .page-header p {
    margin: 0;
    font-size: 0.9rem;
  }
  .search-wrap {
    position: relative;
    min-width: 300px;
    flex-shrink: 0;
    padding-top: 4px;
  }
  .search-wrap input {
    width: 100%;
  }
  .dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: var(--bg-elev);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow);
    z-index: 20;
    max-height: 300px;
    overflow-y: auto;
  }
  .dropdown-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    padding: 8px 12px;
    background: none;
    border: none;
    border-bottom: 1px solid var(--line-soft);
    color: var(--ink);
    text-align: left;
    cursor: pointer;
    font-size: 0.9rem;
  }
  .dropdown-item:hover {
    background: var(--bg-soft);
    transform: none;
  }
  .dropdown-item:last-child {
    border-bottom: none;
  }
  .meta {
    font-size: 0.78rem;
    flex-shrink: 0;
  }
  .content-row {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 16px;
    margin-top: 12px;
    align-items: start;
  }
  .bad {
    color: var(--bad);
  }
  @media (max-width: 900px) {
    .page-header {
      flex-direction: column;
    }
    .content-row {
      grid-template-columns: 1fr;
    }
  }
</style>
