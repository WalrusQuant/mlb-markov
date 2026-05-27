<script lang="ts">
  import type { StateExpectedRuns } from "$lib/types";

  let { expectedRuns }: { expectedRuns: StateExpectedRuns[] } = $props();

  let sorted = $derived(
    [...expectedRuns].sort((a, b) => b.expectedRuns - a.expectedRuns),
  );
</script>

<div class="card">
  <h3>Expected Runs (RE24)</h3>
  <p class="muted">Average runs scored from each state until end of inning.</p>
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th>State</th>
          <th>Expected Runs</th>
        </tr>
      </thead>
      <tbody>
        {#each sorted as row}
          <tr>
            <td>{row.label}</td>
            <td class="mono">{row.expectedRuns.toFixed(3)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

<style>
  .table-wrap {
    max-height: 600px;
    overflow-y: auto;
  }
  td.mono {
    font-weight: 600;
  }
</style>
