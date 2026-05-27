<script lang="ts">
  import type { StateExpectedRuns } from "$lib/types";

  let {
    coldRuns,
    hotRuns,
  }: {
    coldRuns: StateExpectedRuns[];
    hotRuns: StateExpectedRuns[];
  } = $props();

  let rows = $derived.by(() => {
    const coldMap = new Map(coldRuns.map((r) => [r.state, r]));
    return hotRuns.map((h) => {
      const c = coldMap.get(h.state);
      return {
        label: h.label,
        coldER: c?.expectedRuns ?? 0,
        hotER: h.expectedRuns,
        delta: h.expectedRuns - (c?.expectedRuns ?? 0),
      };
    }).sort((a, b) => Math.abs(b.delta) - Math.abs(a.delta));
  });

  let maxDelta = $derived(
    Math.max(...rows.map((r) => Math.abs(r.delta)), 0.01),
  );

  function intensity(delta: number): number {
    return Math.min(Math.abs(delta) / maxDelta, 1) * 60;
  }
</script>

<div class="wrapper">
  <div class="table-head">
    <h3>Cold Innings vs Hot Innings</h3>
    <p class="sub muted">Expected runs from each situation when 0 runs have scored (cold) vs after runs have already scored (hot).</p>
  </div>
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th>Situation</th>
          <th>Cold</th>
          <th>Hot</th>
          <th>Diff</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row}
          <tr>
            <td>{row.label}</td>
            <td class="mono">{row.coldER.toFixed(3)}</td>
            <td class="mono">{row.hotER.toFixed(3)}</td>
            <td
              class="mono delta"
              style="color: {row.delta >= 0 ? 'var(--good)' : 'var(--bad)'}; background: color-mix(in srgb, {row.delta >= 0 ? 'var(--good-soft)' : 'var(--accent-soft)'} {intensity(row.delta)}%, transparent)"
            >
              {row.delta >= 0 ? "+" : ""}{row.delta.toFixed(3)}
            </td>
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
  .delta {
    font-weight: 700;
    border-radius: 2px;
  }
</style>
