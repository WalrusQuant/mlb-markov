<script lang="ts">
  import { renderHeatmap } from "$lib/charts/heatmap";
  import type { StateExpectedRuns } from "$lib/types";

  let {
    states,
    matrix,
    labels,
    expectedRuns,
  }: {
    states: string[];
    matrix: number[][];
    labels?: string[];
    expectedRuns?: StateExpectedRuns[];
  } = $props();

  let container: HTMLElement;

  $effect(() => {
    if (container && states.length > 0 && matrix.length > 0) {
      const size = container.clientWidth;
      if (size > 0) {
        let erMap: Map<string, number> | undefined;
        if (expectedRuns) {
          erMap = new Map(expectedRuns.map((r) => [r.state, r.expectedRuns]));
        }
        renderHeatmap(container, states, matrix, labels, {
          width: size,
          height: size,
          margin: { top: 50, right: 10, bottom: 10, left: 55 },
          expectedRuns: erMap,
        });
      }
    }
  });
</script>

<div bind:this={container} class="heatmap-container"></div>

<style>
  .heatmap-container {
    width: 100%;
    position: relative;
  }
</style>
