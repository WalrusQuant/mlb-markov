<script lang="ts">
  import { onMount } from "svelte";
  import { getOffenseTransitions, getTeams } from "$lib/api";
  import type { OffenseBundle, TeamOption } from "$lib/types";
  import StateHeatmap from "$lib/components/StateHeatmap.svelte";
  import ExpectedRunsTable from "$lib/components/ExpectedRunsTable.svelte";
  import TeamSelector from "$lib/components/TeamSelector.svelte";

  let bundle = $state<OffenseBundle | null>(null);
  let teams = $state<TeamOption[]>([]);
  let selectedTeam = $state<number | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let season = 2024;

  async function load() {
    loading = true;
    error = null;
    try {
      bundle = await getOffenseTransitions(season, selectedTeam);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function onTeamChange() {
    await load();
  }

  onMount(async () => {
    try {
      teams = await getTeams();
    } catch (_) {}
    await load();
  });
</script>

<div class="header-row">
  <div>
    <h1>Offense: At-Bat Transitions</h1>
    <p>25 base-out state transition probabilities and expected runs from any state.</p>
  </div>
  <div class="controls">
    <TeamSelector {teams} bind:value={selectedTeam} onchange={onTeamChange} />
  </div>
</div>

{#if error}
  <div class="card">
    <p class="bad">{error}</p>
    <p class="muted">Import play-by-play data from the Home page to populate this view.</p>
  </div>
{:else if loading}
  <div class="card">
    <p class="muted">Computing transition matrix...</p>
  </div>
{:else if bundle}
  <div class="content-grid">
    <div class="main-col">
      <div class="card">
        <h3>Transition Heatmap</h3>
        <p class="muted">
          Probability of moving from one state (row) to another (column) on each plate appearance.
        </p>
        <StateHeatmap states={bundle.states} matrix={bundle.matrix} />
      </div>
    </div>
    <div class="side-col">
      <ExpectedRunsTable expectedRuns={bundle.expectedRuns} />
    </div>
  </div>
{/if}

<style>
  .header-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 20px;
    gap: 16px;
  }
  .controls {
    flex-shrink: 0;
    padding-top: 8px;
  }
  .content-grid {
    display: grid;
    grid-template-columns: 1fr 340px;
    gap: 20px;
    align-items: start;
  }
  .bad {
    color: var(--bad);
  }
  @media (max-width: 1100px) {
    .content-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
