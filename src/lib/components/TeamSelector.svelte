<script lang="ts">
  import type { TeamOption } from "$lib/types";

  let {
    teams,
    value = $bindable<number | null>(null),
    onchange,
  }: { teams: TeamOption[]; value: number | null; onchange?: () => void } = $props();

  function handleChange(e: Event) {
    const select = e.target as HTMLSelectElement;
    value = select.value === "" ? null : Number(select.value);
    onchange?.();
  }
</script>

<select value={value ?? ""} onchange={handleChange}>
  <option value="">League Average</option>
  {#each teams as team}
    <option value={team.teamId}>{team.name}</option>
  {/each}
</select>
