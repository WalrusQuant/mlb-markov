<script lang="ts">
  import { renderHeatmap } from "$lib/charts/heatmap";
  import type { PitchMatrixData } from "$lib/types";

  let { data, title = "Pitch Transition Matrix" }: { data: PitchMatrixData; title?: string } =
    $props();

  let wrapper: HTMLElement;
  let container: HTMLElement;

  $effect(() => {
    if (container && wrapper && data.types.length > 0) {
      const w = wrapper.clientWidth;
      const h = wrapper.clientHeight;
      const size = Math.min(w, h);
      if (size > 0) {
        renderHeatmap(container, data.types, data.matrix, undefined, {
          width: size,
          height: size,
          margin: { top: 35, right: 10, bottom: 10, left: 45 },
        });
      }
    }
  });
</script>

<div class="card matrix-card">
  <h3>{title}</h3>
  <p class="sub muted">Row = last pitch. Column = next pitch. Hover for details.</p>
  <div bind:this={wrapper} class="matrix-wrap">
    <div bind:this={container} class="matrix-inner"></div>
  </div>
</div>

<style>
  .matrix-card {
    padding: 10px;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .matrix-card h3 {
    margin: 0 0 2px;
    font-size: 0.95rem;
    flex-shrink: 0;
  }
  .sub {
    font-size: 0.72rem;
    margin: 0 0 6px;
    line-height: 1.3;
    flex-shrink: 0;
  }
  .matrix-wrap {
    flex: 1;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .matrix-inner {
    position: relative;
  }
</style>
