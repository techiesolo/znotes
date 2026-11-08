<script lang="ts">
  import { settings } from "../state/settings.svelte";
  import Modal from "./Modal.svelte";

  interface Props {
    onclose: () => void;
  }

  let { onclose }: Props = $props();

  let minutes = $state(String(settings.current.autolock_minutes));
  let localError = $state<string | null>(null);

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    const value = Number(minutes);
    if (!Number.isInteger(value) || value < 1 || value > 120) {
      localError = "Auto-lock must be a whole number between 1 and 120 minutes";
      return;
    }
    localError = null;
    if (await settings.save({ autolock_minutes: value })) onclose();
  }
</script>

<Modal title="Settings" {onclose}>
  <form onsubmit={submit}>
    <label>
      Auto-lock after inactivity (minutes)
      <input type="number" min="1" max="120" step="1" bind:value={minutes} aria-label="Auto-lock minutes" />
    </label>
    <p class="note">While locked, the vault is closed and the key is wiped from memory. Screenshots or keyloggers on this machine are not something the app can defend against.</p>
    {#if localError || settings.error}
      <p class="error" role="alert">{localError ?? settings.error}</p>
    {/if}
    <div class="actions">
      <button type="button" onclick={onclose}>Cancel</button>
      <button type="submit" class="primary">Save</button>
    </div>
  </form>
</Modal>

<style>
  form {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
</style>
