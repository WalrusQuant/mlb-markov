<script lang="ts">
  import type { CountEntropyData, PitchMatrixData } from "$lib/types";

  let {
    countEntropy,
    byCount,
  }: {
    countEntropy: CountEntropyData[];
    byCount: Record<string, PitchMatrixData>;
  } = $props();

  let sorted = $derived(
    [...countEntropy].sort((a, b) => a.countState.localeCompare(b.countState)),
  );

  let maxEntropy = $derived(
    Math.max(...countEntropy.map((c) => c.entropy), 0.1),
  );
</script>

<div class="card">
  <h3>Entropy by Count</h3>
  <p class="muted">How predictable is this pitcher at each count?</p>

  <div class="count-grid">
    {#each sorted as ce}
      <div class="count-item">
        <div class="count-header">
          <span class="mono count-label">{ce.countState}</span>
          <span class="mono entropy">{ce.entropy.toFixed(3)}</span>
        </div>
        <div class="mini-bar">
          <div
            class="mini-fill"
            style="width: {(ce.entropy / maxEntropy) * 100}%"
          ></div>
        </div>
        <span class="pitch-count muted">{ce.pitches} transitions</span>
      </div>
    {/each}
  </div>
</div>

<style>
  .count-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 12px;
    margin-top: 12px;
  }
  .count-item {
    padding: 10px;
    background: var(--bg-soft);
    border-radius: var(--radius-sm);
  }
  .count-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 6px;
  }
  .count-label {
    font-weight: 700;
    font-size: 1rem;
  }
  .entropy {
    font-size: 0.85rem;
    color: var(--ink-soft);
  }
  .mini-bar {
    height: 6px;
    background: var(--line-soft);
    border-radius: 3px;
    overflow: hidden;
  }
  .mini-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 3px;
    transition: width 200ms ease;
  }
  .pitch-count {
    display: block;
    font-size: 0.72rem;
    margin-top: 4px;
  }
</style>
