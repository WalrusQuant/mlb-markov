<script lang="ts">
  import { searchPitchers, getPitchSequences } from "$lib/api";
  import type { PitcherSearchResult, PitchSequenceBundle, PitchMatrixData } from "$lib/types";
  import PitchMatrix from "$lib/components/PitchMatrix.svelte";

  let query = $state("");
  let results = $state<PitcherSearchResult[]>([]);
  let bundle = $state<PitchSequenceBundle | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let selectedCount = $state<string | null>(null);

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  let allCounts = $derived.by(() => {
    if (!bundle) return [];
    return Object.keys(bundle.byCount).sort();
  });

  let activeMatrix = $derived.by((): PitchMatrixData | null => {
    if (!bundle) return null;
    if (selectedCount && bundle.byCount[selectedCount]) {
      return bundle.byCount[selectedCount];
    }
    return bundle.overallMatrix;
  });

  let activeLabel = $derived(selectedCount ?? "Overall");

  let activeEntropy = $derived.by(() => {
    if (!bundle) return null;
    if (selectedCount) {
      const ce = bundle.countEntropy.find((c) => c.countState === selectedCount);
      return ce ?? null;
    }
    return { countState: "Overall", entropy: bundle.overallEntropy, pitches: bundle.totalPitches };
  });

  let maxEntropy = $derived(
    bundle ? Math.log(Math.max(bundle.pitchTypes.length, 2)) : 1,
  );

  let predictPct = $derived(
    activeEntropy ? (activeEntropy.entropy / maxEntropy) * 100 : 0,
  );

  let predictLabel = $derived(
    predictPct > 70 ? "Unpredictable" : predictPct > 40 ? "Moderate" : "Predictable",
  );

  function countTransitions(count: string): number {
    const mat = bundle?.byCount[count];
    if (!mat) return 0;
    return mat.occurrences.flat().reduce((a, b) => a + b, 0);
  }

  function onInput() {
    if (debounceTimer) clearTimeout(debounceTimer);
    if (query.length < 2) {
      results = [];
      return;
    }
    debounceTimer = setTimeout(async () => {
      try {
        results = await searchPitchers(query);
      } catch (_) {
        results = [];
      }
    }, 300);
  }

  async function selectPitcher(pitcher: PitcherSearchResult) {
    query = pitcher.fullName;
    results = [];
    loading = true;
    error = null;
    selectedCount = null;
    try {
      bundle = await getPitchSequences(pitcher.playerId);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
</script>

<div class="page">
  <div class="header-row">
    <div>
      <h1>Pitching: What's Coming Next?</h1>
      <p class="subtitle">Pick a pitcher and a count to see what they throw — and what follows each pitch type.</p>
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
    <div class="card"><p class="bad">{error}</p></div>
  {:else if loading}
    <div class="card"><p class="muted">Loading...</p></div>
  {:else if bundle}
    <div class="content">
      <div class="left-col">
        <div class="pitcher-info card">
          <div class="pitcher-top">
            <h2>{bundle.pitcherName}</h2>
            <div class="badges">
              <span class="badge">{bundle.totalPitches.toLocaleString()} pitches</span>
              <span class="badge">{bundle.pitchTypes.join(", ")}</span>
            </div>
          </div>
        </div>

        <div class="count-selector card">
          <h3>Select a count</h3>
          <div class="count-grid">
            <button
              class="count-btn"
              class:active={selectedCount === null}
              onclick={() => (selectedCount = null)}
            >
              All
            </button>
            {#each allCounts as count}
              {@const transitions = countTransitions(count)}
              <button
                class="count-btn"
                class:active={selectedCount === count}
                class:dim={transitions < 20}
                onclick={() => (selectedCount = count)}
              >
                <span class="count-label">{count}</span>
                <span class="count-n muted">{transitions}</span>
              </button>
            {/each}
          </div>
        </div>

        {#if activeEntropy}
          <div class="predict-card card">
            <div class="predict-row">
              <span class="predict-label">{activeLabel} predictability</span>
              <span class="mono predict-value">{activeEntropy.entropy.toFixed(3)}</span>
            </div>
            <div class="predict-bar">
              <div class="predict-fill" style="width: {predictPct}%"></div>
            </div>
            <div class="predict-labels">
              <span class="muted">Predictable</span>
              <span class="predict-interp">{predictLabel}</span>
              <span class="muted">Random</span>
            </div>
          </div>
        {/if}
      </div>

      <div class="right-col">
        {#if activeMatrix}
          <PitchMatrix
            data={activeMatrix}
            title="{activeLabel === 'Overall' ? 'Overall' : 'At ' + activeLabel}: After each pitch, what comes next?"
          />
        {/if}
      </div>
    </div>
  {:else}
    <div class="card">
      <p class="muted">Import play-by-play data from the Home page, then search for a pitcher above.</p>
    </div>
  {/if}
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: calc(100vh - 105px);
    overflow: hidden;
  }
  .header-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 16px;
    flex-shrink: 0;
  }
  .header-row h1 {
    font-size: 1.3rem;
    margin: 0;
  }
  .subtitle {
    font-size: 0.8rem;
    margin: 2px 0 0;
    line-height: 1.3;
  }
  .search-wrap {
    position: relative;
    min-width: 280px;
    flex-shrink: 0;
    padding-top: 2px;
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
    max-height: 250px;
    overflow-y: auto;
  }
  .dropdown-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    padding: 6px 10px;
    background: none;
    border: none;
    border-bottom: 1px solid var(--line-soft);
    color: var(--ink);
    text-align: left;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .dropdown-item:hover {
    background: var(--bg-soft);
    transform: none;
  }
  .dropdown-item:last-child {
    border-bottom: none;
  }
  .meta {
    font-size: 0.75rem;
    flex-shrink: 0;
  }

  .content {
    display: flex;
    gap: 10px;
    flex: 1;
    min-height: 0;
    margin-top: 8px;
  }
  .left-col {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 340px;
    flex-shrink: 0;
  }
  .right-col {
    flex: 1;
    min-width: 0;
    height: 100%;
  }

  .pitcher-info {
    padding: 10px;
    flex-shrink: 0;
  }
  .pitcher-info h2 {
    margin: 0;
    font-size: 1.1rem;
  }
  .pitcher-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
  }
  .badges {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
  }

  .count-selector {
    padding: 10px;
    flex-shrink: 0;
  }
  .count-selector h3 {
    margin: 0 0 6px;
    font-size: 0.85rem;
  }
  .count-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .count-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 4px 8px;
    font-size: 0.75rem;
    background: var(--bg-soft);
    border: 1px solid var(--line-soft);
    color: var(--ink);
    border-radius: var(--radius-sm);
    cursor: pointer;
    min-width: 44px;
  }
  .count-btn:hover {
    background: var(--bg-elev);
    transform: none;
  }
  .count-btn.active {
    background: var(--ink);
    color: var(--bg-elev);
    border-color: var(--ink);
  }
  .count-btn.dim {
    opacity: 0.5;
  }
  .count-label {
    font-weight: 700;
    font-family: var(--mono);
  }
  .count-n {
    font-size: 0.65rem;
    font-family: var(--mono);
  }

  .predict-card {
    padding: 10px;
    flex-shrink: 0;
  }
  .predict-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 4px;
  }
  .predict-label {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--ink-soft);
  }
  .predict-value {
    font-size: 1rem;
    font-weight: 700;
  }
  .predict-bar {
    height: 8px;
    background: var(--bg-soft);
    border-radius: 4px;
    overflow: hidden;
  }
  .predict-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--bad), var(--warn), var(--good));
    border-radius: 4px;
    transition: width 200ms ease;
  }
  .predict-labels {
    display: flex;
    justify-content: space-between;
    font-size: 0.68rem;
    margin-top: 3px;
  }
  .predict-interp {
    font-weight: 600;
    color: var(--ink);
  }

  .bad {
    color: var(--bad);
  }
</style>
