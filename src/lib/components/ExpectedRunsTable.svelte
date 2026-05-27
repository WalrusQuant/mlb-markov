<script lang="ts">
  import type { StateExpectedRuns } from "$lib/types";

  let { expectedRuns }: { expectedRuns: StateExpectedRuns[] } = $props();

  let sorted = $derived(
    [...expectedRuns].sort((a, b) => b.expectedRuns - a.expectedRuns),
  );
</script>

<div class="card wrapper">
  <h3>Run Expectancy (RE24)</h3>
  <p class="sub muted">When teams are in this situation, this is how many runs they actually score before the inning ends.</p>
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th>Situation</th>
          <th>Exp. Runs</th>
          <th>Freq</th>
        </tr>
      </thead>
      <tbody>
        {#each sorted as row}
          <tr>
            <td>{row.label}</td>
            <td class="mono er">{row.expectedRuns.toFixed(3)}</td>
            <td class="mono freq">{(row.frequency * 100).toFixed(1)}%</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

<style>
  .wrapper {
    padding: 10px;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .wrapper h3 {
    margin: 0 0 1px;
    font-size: 0.95rem;
    flex-shrink: 0;
  }
  .sub {
    font-size: 0.72rem;
    margin: 0 0 6px;
    flex-shrink: 0;
    line-height: 1.3;
  }
  .table-wrap {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }
  table {
    font-size: 0.78rem;
    width: 100%;
  }
  th, td {
    padding: 3px 6px;
  }
  th {
    position: sticky;
    top: 0;
    background: var(--bg-elev);
  }
  .er {
    font-weight: 600;
  }
  .freq {
    color: var(--ink-soft);
  }
</style>
