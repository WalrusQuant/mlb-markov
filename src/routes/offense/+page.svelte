<script lang="ts">
  import { onMount } from "svelte";
  import { getOffenseTransitions, getTeams, getMomentumAnalysis } from "$lib/api";
  import type { OffenseBundle, TeamOption, RunComparison, MomentumBundle } from "$lib/types";
  import StateHeatmap from "$lib/components/StateHeatmap.svelte";
  import ExpectedRunsTable from "$lib/components/ExpectedRunsTable.svelte";
  import RunComparisonTable from "$lib/components/RunComparisonTable.svelte";
  import InsightCallouts from "$lib/components/InsightCallouts.svelte";
  import MomentumTable from "$lib/components/MomentumTable.svelte";
  import TeamSelector from "$lib/components/TeamSelector.svelte";

  let viewMode = $state<"edge" | "momentum">("edge");
  let leagueBundle = $state<OffenseBundle | null>(null);
  let teamBundle = $state<OffenseBundle | null>(null);
  let momentumBundle = $state<MomentumBundle | null>(null);
  let teams = $state<TeamOption[]>([]);
  let selectedTeam = $state<number | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);

  let selectedTeamName = $derived(
    teams.find((t) => t.teamId === selectedTeam)?.name ?? "",
  );

  let comparisonRows: RunComparison[] = $derived.by(() => {
    if (!teamBundle || !leagueBundle) return [];
    const leagueMap = new Map(
      leagueBundle.expectedRuns.map((r) => [r.state, r]),
    );
    return teamBundle.expectedRuns.map((t) => {
      const l = leagueMap.get(t.state);
      return {
        state: t.state,
        label: t.label,
        teamER: t.expectedRuns,
        leagueER: l?.expectedRuns ?? 0,
        delta: t.expectedRuns - (l?.expectedRuns ?? 0),
        frequency: t.frequency,
      };
    });
  });

  let hasTeam = $derived(selectedTeam !== null && teamBundle !== null);

  async function loadEdge() {
    loading = true;
    error = null;
    try {
      if (!leagueBundle) {
        leagueBundle = await getOffenseTransitions(undefined, null);
      }
      if (selectedTeam) {
        teamBundle = await getOffenseTransitions(undefined, selectedTeam);
      } else {
        teamBundle = null;
      }
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function loadMomentum() {
    loading = true;
    error = null;
    try {
      momentumBundle = await getMomentumAnalysis(undefined, selectedTeam);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function onTeamChange() {
    momentumBundle = null;
    if (viewMode === "edge") await loadEdge();
    else await loadMomentum();
  }

  async function switchView(mode: "edge" | "momentum") {
    viewMode = mode;
    if (mode === "momentum" && !momentumBundle) {
      await loadMomentum();
    } else if (mode === "edge" && !leagueBundle) {
      await loadEdge();
    }
  }

  onMount(async () => {
    try {
      teams = await getTeams();
    } catch (_) {}
    await loadEdge();
  });
</script>

<div class="page">
  <div class="header-row">
    <div class="header-left">
      <h1>{viewMode === "edge" ? "Offense: Find the Edge" : "Offense: Does Momentum Matter?"}</h1>
      <p class="subtitle">
        {#if viewMode === "edge"}
          Select a team to see where they score more or fewer runs than league average.
        {:else}
          Do teams score more from the same situation when runs have already scored in the inning?
        {/if}
      </p>
    </div>
    <div class="controls">
      <div class="tabs">
        <button class:active={viewMode === "edge"} onclick={() => switchView("edge")}>Team Edge</button>
        <button class:active={viewMode === "momentum"} onclick={() => switchView("momentum")}>Momentum</button>
      </div>
      <TeamSelector {teams} bind:value={selectedTeam} onchange={onTeamChange} />
    </div>
  </div>

  {#if error}
    <div class="card"><p class="bad">{error}</p></div>
  {:else if loading}
    <div class="card"><p class="muted">Computing...</p></div>

  {:else if viewMode === "momentum" && momentumBundle}
    <div class="verdict-row">
      <div class="verdict-pill" class:good={momentumBundle.overallDelta > 0.05} class:bad={momentumBundle.overallDelta <= 0.05}>
        <strong>{momentumBundle.verdict}</strong>
      </div>
      <span class="sample-size muted">
        {momentumBundle.coldTotalPlays.toLocaleString()} cold plays · {momentumBundle.hotTotalPlays.toLocaleString()} hot plays
      </span>
    </div>
    <div class="content-grid">
      <div class="table-col card">
        <MomentumTable coldRuns={momentumBundle.coldExpectedRuns} hotRuns={momentumBundle.hotExpectedRuns} />
      </div>
      <div class="heatmap-col card compact">
        <StateHeatmap
          states={momentumBundle.states}
          matrix={momentumBundle.hotMatrix}
          expectedRuns={momentumBundle.hotExpectedRuns}
        />
      </div>
    </div>

  {:else if viewMode === "edge" && hasTeam && comparisonRows.length > 0}
    <div class="insights-row">
      <InsightCallouts rows={comparisonRows} teamName={selectedTeamName} />
    </div>
    <div class="content-grid">
      <div class="table-col card">
        <RunComparisonTable rows={comparisonRows} teamName={selectedTeamName} />
      </div>
      <div class="heatmap-col card compact">
        <StateHeatmap states={teamBundle?.states ?? []} matrix={teamBundle?.matrix ?? []} expectedRuns={teamBundle?.expectedRuns} />
      </div>
    </div>

  {:else if viewMode === "edge" && leagueBundle}
    <div class="content-grid">
      <div class="table-col card">
        <ExpectedRunsTable expectedRuns={leagueBundle.expectedRuns} />
      </div>
      <div class="heatmap-col card compact">
        <StateHeatmap states={leagueBundle.states} matrix={leagueBundle.matrix} expectedRuns={leagueBundle.expectedRuns} />
      </div>
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
    max-width: 520px;
  }
  .controls {
    display: flex;
    gap: 10px;
    align-items: center;
    flex-shrink: 0;
    padding-top: 2px;
  }
  .tabs {
    display: flex;
    gap: 0;
  }
  .tabs button {
    font-size: 0.78rem;
    padding: 4px 12px;
    background: transparent;
    color: var(--ink-soft);
    border: 1px solid var(--line);
    cursor: pointer;
  }
  .tabs button:first-child {
    border-radius: var(--radius-sm) 0 0 var(--radius-sm);
  }
  .tabs button:last-child {
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
    border-left: none;
  }
  .tabs button.active {
    background: var(--ink);
    color: var(--bg-elev);
    border-color: var(--ink);
  }
  .tabs button:hover:not(.active) {
    background: var(--bg-soft);
    transform: none;
  }

  .verdict-row {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 6px 0;
    flex-shrink: 0;
  }
  .verdict-pill {
    padding: 4px 12px;
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
  }
  .verdict-pill.good {
    background: var(--good-soft);
    color: var(--good);
  }
  .verdict-pill.bad {
    background: var(--accent-soft);
    color: var(--bad);
  }
  .sample-size {
    font-size: 0.72rem;
  }

  .insights-row {
    flex-shrink: 0;
    margin: 6px 0;
  }

  .compact {
    padding: 8px;
  }
  .content-grid {
    display: flex;
    gap: 10px;
    flex: 1;
    min-height: 0;
    margin-top: 6px;
  }
  .table-col {
    flex: 1;
    min-width: 0;
    height: 100%;
    overflow: hidden;
    padding: 10px;
  }
  .heatmap-col {
    flex-shrink: 0;
    width: calc(100vh - 210px);
    max-width: 50%;
    height: 100%;
  }
  .bad {
    color: var(--bad);
  }
  @media (max-width: 900px) {
    .content-grid {
      flex-direction: column;
    }
    .heatmap-col {
      width: 100%;
    }
  }
</style>
