<script lang="ts">
  import { save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { api, errorMessage, type Note } from "../api";
  import { suggestedExportName } from "../utils/filename";
  import Modal from "./Modal.svelte";

  interface Props {
    note: Note;
    onclose: () => void;
  }

  let { note, onclose }: Props = $props();

  let error = $state<string | null>(null);
  let exportedTo = $state<string | null>(null);

  async function pickAndExport() {
    error = null;
    const path = await saveDialog({
      title: "Export note as Markdown (unencrypted)",
      defaultPath: suggestedExportName(note.title),
      filters: [{ name: "Markdown", extensions: ["md"] }],
    });
    if (!path) return;
    try {
      await api.noteExportMd(note.id, path);
      exportedTo = path;
    } catch (err) {
      error = errorMessage(err);
    }
  }
</script>

<Modal title="Export note" {onclose}>
  {#if exportedTo}
    <p>Exported to <code>{exportedTo}</code>.</p>
    <div class="actions">
      <button type="button" class="primary" onclick={onclose}>Done</button>
    </div>
  {:else}
    <p><strong>This writes an unencrypted file.</strong> Anyone or anything that can read that location can read the note. Delete it when you no longer need it.</p>
    {#if error}
      <p class="error" role="alert">{error}</p>
    {/if}
    <div class="actions">
      <button type="button" onclick={onclose}>Cancel</button>
      <button type="button" class="danger" onclick={pickAndExport}>Choose location and export</button>
    </div>
  {/if}
</Modal>

<style>
  code {
    word-break: break-all;
  }
</style>
