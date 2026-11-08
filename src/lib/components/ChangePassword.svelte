<script lang="ts">
  import { api, errorMessage, isAppError } from "../api";
  import Modal from "./Modal.svelte";

  interface Props {
    onclose: () => void;
  }

  let { onclose }: Props = $props();

  let current = $state("");
  let next = $state("");
  let confirm = $state("");
  let error = $state<string | null>(null);
  let busy = $state(false);
  let done = $state(false);

  const canSubmit = $derived(current.length > 0 && next.length > 0 && next === confirm && !busy);

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (next !== confirm) {
      error = "New passwords do not match";
      return;
    }
    busy = true;
    error = null;
    try {
      await api.vaultChangePassword(current, next);
      done = true;
    } catch (err) {
      error = isAppError(err) && err.code === "wrong_password" ? "Current password is wrong" : errorMessage(err);
      current = "";
    } finally {
      busy = false;
    }
  }
</script>

<Modal title="Change password" {onclose}>
  {#if done}
    <p>Password changed. Use the new password next time you unlock.</p>
    <div class="actions">
      <button type="button" class="primary" onclick={onclose}>Done</button>
    </div>
  {:else}
    <form onsubmit={submit}>
      <label>
        Current password
        <input type="password" bind:value={current} autocomplete="off" />
      </label>
      <label>
        New password
        <input type="password" bind:value={next} autocomplete="off" />
      </label>
      <label>
        Confirm new password
        <input type="password" bind:value={confirm} autocomplete="off" />
      </label>
      <p class="note">The whole vault is re-encrypted with the new password. There is no recovery if you forget it.</p>
      {#if error}
        <p class="error" role="alert">{error}</p>
      {/if}
      <div class="actions">
        <button type="button" onclick={onclose}>Cancel</button>
        <button type="submit" class="primary" disabled={!canSubmit}>{busy ? "Changing…" : "Change password"}</button>
      </div>
    </form>
  {/if}
</Modal>

<style>
  form {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
</style>
