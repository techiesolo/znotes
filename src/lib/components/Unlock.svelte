<script lang="ts">
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { vault } from "../state/vault.svelte";
  import { settings } from "../state/settings.svelte";

  type Mode = "open" | "create";

  let mode = $state<Mode>("open");
  let dir = $state(vault.dir ?? settings.current.last_vault_dir ?? "");
  let password = $state("");
  let confirm = $state("");
  let localError = $state<string | null>(null);
  let passwordInput: HTMLInputElement | undefined = $state();

  const canSubmit = $derived(
    dir.trim().length > 0 && password.length > 0 && (mode === "open" || confirm === password),
  );

  async function chooseDir() {
    const picked = await openDialog({ directory: true, multiple: false, title: "Choose vault folder" });
    if (typeof picked === "string") dir = picked;
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    localError = null;
    if (mode === "create" && confirm !== password) {
      localError = "Passwords do not match";
      return;
    }
    const ok =
      mode === "open" ? await vault.open(dir.trim(), password) : await vault.create(dir.trim(), password);
    if (!ok) {
      password = "";
      confirm = "";
      passwordInput?.focus();
    }
  }

  function switchMode(next: Mode) {
    mode = next;
    localError = null;
    vault.error = null;
  }
</script>

<div class="unlock">
  <form class="card" onsubmit={submit}>
    <h1>znotes</h1>
    <p class="hint">
      {#if vault.lockReason === "idle"}
        Locked after inactivity. Enter your password to continue.
      {:else}
        Your notes are encrypted on disk. Nothing is readable without the password.
      {/if}
    </p>

    <div class="tabs" role="tablist">
      <button type="button" role="tab" aria-selected={mode === "open"} class:active={mode === "open"} onclick={() => switchMode("open")}>
        Open vault
      </button>
      <button type="button" role="tab" aria-selected={mode === "create"} class:active={mode === "create"} onclick={() => switchMode("create")}>
        Create vault
      </button>
    </div>

    <div class="field">
      <label for="vault-dir">Vault folder</label>
      <div class="row">
        <input id="vault-dir" type="text" bind:value={dir} placeholder="/path/to/vault" autocomplete="off" spellcheck="false" />
        <button type="button" onclick={chooseDir}>Choose…</button>
      </div>
    </div>

    <label>
      <span>Password</span>
      <input
        type="password"
        bind:value={password}
        bind:this={passwordInput}
        autocomplete="off"
        aria-label="Password"
      />
    </label>

    {#if mode === "create"}
      <label>
        <span>Confirm password</span>
        <input type="password" bind:value={confirm} autocomplete="off" aria-label="Confirm password" />
      </label>
      <p class="warn">If you forget this password, the vault cannot be recovered.</p>
    {/if}

    {#if localError || vault.error}
      <p class="error" role="alert">{localError ?? vault.error}</p>
    {/if}

    <button type="submit" class="primary" disabled={!canSubmit || vault.busy}>
      {#if vault.busy}
        Unlocking…
      {:else if mode === "open"}
        Unlock
      {:else}
        Create and unlock
      {/if}
    </button>
  </form>
</div>

<style>
  .unlock {
    height: 100%;
    display: grid;
    place-items: center;
    background: var(--bg-muted);
  }
  .card {
    width: min(420px, 90vw);
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 28px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  h1 {
    margin: 0;
    font-size: 22px;
  }
  .hint {
    margin: 0;
    color: var(--fg-muted);
  }
  .tabs {
    display: flex;
    gap: 4px;
    border-bottom: 1px solid var(--border);
  }
  .tabs button {
    background: none;
    border: none;
    padding: 8px 12px;
    cursor: pointer;
    color: var(--fg-muted);
    border-bottom: 2px solid transparent;
  }
  .tabs button.active {
    color: var(--fg);
    border-bottom-color: var(--accent);
  }
  label,
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  label span,
  .field label {
    font-size: 12px;
    color: var(--fg-muted);
  }
  .row {
    display: flex;
    gap: 6px;
  }
  .row input {
    flex: 1;
  }
  input {
    padding: 8px 10px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg);
  }
  button {
    padding: 8px 12px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-muted);
    cursor: pointer;
  }
  button.primary {
    background: var(--accent);
    color: var(--accent-fg);
    border-color: transparent;
  }
  button:disabled {
    opacity: 0.6;
    cursor: default;
  }
  .error {
    margin: 0;
    color: var(--danger);
  }
  .warn {
    margin: 0;
    font-size: 12px;
    color: var(--fg-muted);
  }
</style>
