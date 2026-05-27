<script lang="ts">
  import { renderHeatmap } from "$lib/charts/heatmap";
  import type { PitchMatrixData } from "$lib/types";

  let { data, title = "Pitch Transition Matrix" }: { data: PitchMatrixData; title?: string } =
    $props();

  let container: HTMLElement;

  $effect(() => {
    if (container && data.types.length > 0) {
      const size = Math.max(300, data.types.length * 70 + 80);
      renderHeatmap(container, data.types, data.matrix, undefined, {
        width: size,
        height: size,
        margin: { top: 40, right: 10, bottom: 10, left: 50 },
      });
    }
  });
</script>

<div class="card">
  <h3>{title}</h3>
  <div bind:this={container} class="pitch-heatmap"></div>
</div>

<style>
  .pitch-heatmap {
    width: 100%;
    max-width: 420px;
  }
</style>
