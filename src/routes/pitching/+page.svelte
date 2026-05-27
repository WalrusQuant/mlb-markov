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

<h1>Pitching: Sequence Predictability</h1>
<p>Search for a pitcher to view their pitch-type transition matrix and predictability score.</p>

<div class="search-area">
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

  <div class="section">
    <PitchMatrix data={bundle.overallMatrix} title="Overall Pitch Transition Matrix" />
  </div>

  {#if bundle.countEntropy.length > 0}
    <div class="section">
      <CountBreakdown countEntropy={bundle.countEntropy} byCount={bundle.byCount} />
    </div>
  {/if}
{:else}
  <div class="card">
    <p class="muted">Import play-by-play data from the Home page, then search for a pitcher above.</p>
  </div>
{/if}

<style>
  .search-area {
    margin-bottom: 24px;
  }
  .search-wrap {
    position: relative;
    max-width: 400px;
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
    padding: 10px 12px;
    background: none;
    border: none;
    border-bottom: 1px solid var(--line-soft);
    color: var(--ink);
    text-align: left;
    cursor: pointer;
    font-size: 0.93rem;
  }
  .dropdown-item:hover {
    background: var(--bg-soft);
    transform: none;
  }
  .dropdown-item:last-child {
    border-bottom: none;
  }
  .meta {
    font-size: 0.8rem;
    flex-shrink: 0;
  }
  .section {
    margin-top: 20px;
  }
  .bad {
    color: var(--bad);
  }
</style>
