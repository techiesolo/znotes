<script lang="ts">
  import { onMount } from "svelte";
  import { vault } from "./lib/state/vault.svelte";
  import Unlock from "./lib/components/Unlock.svelte";
  import Workspace from "./lib/components/Workspace.svelte";

  onMount(() => {
    let unlisten: (() => void) | undefined;
    vault.listenForLock().then((fn) => (unlisten = fn));
    vault.refresh();
    return () => unlisten?.();
  });
</script>

{#if vault.status === "loading"}
  <div class="loading">Loading…</div>
{:else if vault.status === "locked"}
  <Unlock />
{:else}
  <Workspace />
{/if}

<style>
  .loading {
    height: 100%;
    display: grid;
    place-items: center;
    color: var(--fg-muted);
  }
</style>
