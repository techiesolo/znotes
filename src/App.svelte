<script lang="ts">
  import { onMount } from "svelte";
  import { vault } from "./lib/state/vault.svelte";
  import { settings } from "./lib/state/settings.svelte";
  import { workspace } from "./lib/state/workspace.svelte";
  import Unlock from "./lib/components/Unlock.svelte";
  import Workspace from "./lib/components/Workspace.svelte";

  onMount(() => {
    let unlisten: (() => void) | undefined;
    vault.listenForLock().then((fn) => (unlisten = fn));
    settings.load().then(() => {
      workspace.preview = settings.current.preview;
      return vault.refresh();
    });
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
