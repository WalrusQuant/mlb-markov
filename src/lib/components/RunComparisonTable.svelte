<script lang="ts">
  import type { RunComparison } from "$lib/types";

  let { rows, teamName }: { rows: RunComparison[]; teamName: string } = $props();

  let sorted = $derived(
    [...rows].sort((a, b) => Math.abs(b.delta) - Math.abs(a.delta)),
  );

  let maxDelta = $derived(
    Math.max(...rows.map((r) => Math.abs(r.delta)), 0.01),
  );

  function intensity(delta: number): number {
    return Math.min(Math.abs(delta) / maxDelta, 1) * 60;
  }
</script>

<div class="wrapper">
  <div class="table-head">
    <h3>{teamName} vs League Average</h3>
    <p class="sub muted">Sorted by biggest difference. Green = above league average. Red = below.</p>
  </div>
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th>Situation</th>
          <th>Exp. Runs</th>
          <th>Lg Avg</th>
          <th>Diff</th>
          <th>Freq</th>
        </tr>
      </thead>
      <tbody>
        {#each sorted as row}
          <tr>
            <td>{row.label}</td>
            <td class="mono">{row.teamER.toFixed(3)}</td>
            <td class="mono dim">{row.leagueER.toFixed(3)}</td>
            <td
              class="mono delta"
              style="color: {row.delta >= 0 ? 'var(--good)' : 'var(--bad)'}; background: color-mix(in srgb, {row.delta >= 0 ? 'var(--good-soft)' : 'var(--accent-soft)'} {intensity(row.delta)}%, transparent)"
            >
              {row.delta >= 0 ? "+" : ""}{row.delta.toFixed(3)}
            </td>
            <td class="mono dim">{(row.frequency * 100).toFixed(1)}%</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

<style>
  .wrapper {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .table-head {
    flex-shrink: 0;
    margin-bottom: 4px;
  }
  .table-head h3 {
    margin: 0;
    font-size: 0.95rem;
  }
  .sub {
    font-size: 0.7rem;
    margin: 1px 0 0;
    line-height: 1.3;
  }
  .table-wrap {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
    overscroll-behavior: contain;
  }
  table {
    width: 100%;
    font-size: 0.75rem;
  }
  th, :global(td) {
    padding: 2px 6px;
  }
  th {
    position: sticky;
    top: 0;
    background: var(--bg-elev);
    z-index: 1;
  }
  .dim {
    color: var(--ink-mute);
  }
  .delta {
    font-weight: 700;
    border-radius: 2px;
  }
</style>
