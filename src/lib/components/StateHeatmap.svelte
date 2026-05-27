<script lang="ts">
  import { renderHeatmap } from "$lib/charts/heatmap";

  let { states, matrix, labels }: { states: string[]; matrix: number[][]; labels?: string[] } =
    $props();

  let container: HTMLElement;

  $effect(() => {
    if (container && states.length > 0 && matrix.length > 0) {
      renderHeatmap(container, states, matrix, labels);
    }
  });
</script>

<div class="heatmap-wrapper">
  <div bind:this={container} class="heatmap-container"></div>
</div>

<style>
  .heatmap-wrapper {
    overflow-x: auto;
  }
  .heatmap-container {
    min-width: 700px;
  }
</style>
