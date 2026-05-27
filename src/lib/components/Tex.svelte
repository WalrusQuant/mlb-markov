<script lang="ts">
  import katex from "katex";

  let { math, display = false }: { math: string; display?: boolean } = $props();
</script>

{#if display}
  <div class="tex-block">{@html katex.renderToString(math, { displayMode: true, throwOnError: false })}</div>
{:else}
  <span class="tex-inline">{@html katex.renderToString(math, { displayMode: false, throwOnError: false })}</span>
{/if}

<style>
  .tex-block {
    margin: 0.8em 0 1.2em;
    overflow-x: auto;
  }
  .tex-inline {
    vertical-align: baseline;
  }
</style>
