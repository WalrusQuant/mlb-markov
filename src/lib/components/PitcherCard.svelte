<script lang="ts">
  let {
    name,
    team = "",
    entropy,
    totalPitches,
    pitchTypes,
  }: {
    name: string;
    team?: string;
    entropy: number;
    totalPitches: number;
    pitchTypes: string[];
  } = $props();

  let maxEntropy = $derived(Math.log(Math.max(pitchTypes.length, 2)));
  let pct = $derived(maxEntropy > 0 ? (entropy / maxEntropy) * 100 : 0);
  let interpretation = $derived(
    pct > 70 ? "Unpredictable" : pct > 40 ? "Moderate" : "Predictable",
  );
</script>

<div class="card pitcher-card">
  <div class="top-row">
    <div>
      <h2>{name}</h2>
      {#if team}
        <p class="muted team">{team}</p>
      {/if}
    </div>
    <div class="stats">
      <span class="badge">{totalPitches.toLocaleString()} pitches</span>
      <span class="badge">{pitchTypes.length} types</span>
    </div>
  </div>

  <div class="entropy-section">
    <div class="entropy-header">
      <span class="label">Predictability Score</span>
      <span class="mono entropy-value">{entropy.toFixed(3)}</span>
    </div>
    <div class="entropy-bar">
      <div class="entropy-fill" style="width: {pct}%"></div>
    </div>
    <div class="entropy-labels">
      <span class="muted">Predictable</span>
      <span class="interp">{interpretation}</span>
      <span class="muted">Unpredictable</span>
    </div>
  </div>
</div>

<style>
  .pitcher-card h2 {
    margin-bottom: 0;
  }
  .team {
    margin: 0;
    font-size: 0.9rem;
  }
  .top-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 16px;
  }
  .stats {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }
  .entropy-section {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--line-soft);
  }
  .entropy-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 6px;
  }
  .label {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--ink-soft);
  }
  .entropy-value {
    font-size: 1.2rem;
    font-weight: 700;
  }
  .entropy-bar {
    height: 10px;
    background: var(--bg-soft);
    border-radius: 5px;
    overflow: hidden;
  }
  .entropy-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--bad), var(--warn), var(--good));
    border-radius: 5px;
    transition: width 300ms ease;
  }
  .entropy-labels {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    margin-top: 4px;
  }
  .interp {
    font-weight: 600;
    color: var(--ink);
  }
</style>
