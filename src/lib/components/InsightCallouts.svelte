<script lang="ts">
  import type { RunComparison } from "$lib/types";

  let { rows, teamName }: { rows: RunComparison[]; teamName: string } = $props();

  let best = $derived(
    [...rows].sort((a, b) => b.delta - a.delta).slice(0, 2).filter((r) => r.delta > 0.01),
  );
  let worst = $derived(
    [...rows].sort((a, b) => a.delta - b.delta).slice(0, 2).filter((r) => r.delta < -0.01),
  );
</script>

{#if best.length > 0 || worst.length > 0}
  <div class="insights">
    {#each best as b}
      <div class="pill good">
        <span class="arrow">▲</span>
        <span><strong>+{b.delta.toFixed(2)}</strong> runs vs avg — {b.label}</span>
      </div>
    {/each}
    {#each worst as w}
      <div class="pill bad">
        <span class="arrow">▼</span>
        <span><strong>{w.delta.toFixed(2)}</strong> runs vs avg — {w.label}</span>
      </div>
    {/each}
  </div>
{/if}

<style>
  .insights {
    display: flex;
    gap: 8px;
    flex-wrap: nowrap;
    overflow: hidden;
  }
  .pill {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
    line-height: 1.3;
    white-space: nowrap;
    flex-shrink: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .pill.good {
    background: var(--good-soft);
    color: var(--good);
  }
  .pill.bad {
    background: var(--accent-soft);
    color: var(--bad);
  }
  .arrow {
    flex-shrink: 0;
    font-size: 0.7rem;
  }
</style>
