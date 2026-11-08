<script lang="ts">
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { api, errorMessage } from "../api";
  import { vault } from "../state/vault.svelte";
  import { workspace } from "../state/workspace.svelte";
  import { settings } from "../state/settings.svelte";
  import ChangePassword from "./ChangePassword.svelte";
  import SettingsDialog from "./SettingsDialog.svelte";
  import ExportConfirm from "./ExportConfirm.svelte";

  type Dialog = "none" | "password" | "settings" | "export";

  let open = $state(false);
  let dialog = $state<Dialog>("none");
  let toast = $state<string | null>(null);
  let toastTimer: ReturnType<typeof setTimeout> | undefined;

  function show(message: string) {
    toast = message;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toast = null), 6000);
  }

  async function backup() {
    open = false;
    await workspace.flush();
    const dest = await openDialog({ directory: true, multiple: false, title: "Choose backup folder" });
    if (typeof dest !== "string") return;
    try {
      const paths = await api.vaultBackup(dest);
      show(`Encrypted backup written to ${paths.db}`);
    } catch (err) {
      show(`Backup failed: ${errorMessage(err)}`);
    }
  }

  function openDialogOf(kind: Dialog) {
    open = false;
    dialog = kind;
  }

  async function togglePreview() {
    open = false;
    workspace.togglePreview();
    await settings.save({ preview: workspace.preview });
  }
</script>

<div class="menu">
  <button type="button" class="trigger" aria-haspopup="menu" aria-expanded={open} aria-label="More actions" onclick={() => (open = !open)}>⋯</button>
  {#if open}
    <div class="backdrop" role="presentation" onclick={() => (open = false)}></div>
    <ul role="menu">
      <li><button type="button" role="menuitem" onclick={togglePreview}>{workspace.preview ? "Hide preview" : "Show preview"} <kbd>⌘E</kbd></button></li>
      <li><button type="button" role="menuitem" onclick={() => openDialogOf("export")} disabled={!workspace.current}>Export note as .md…</button></li>
      <li class="sep"></li>
      <li><button type="button" role="menuitem" onclick={backup}>Backup vault…</button></li>
      <li><button type="button" role="menuitem" onclick={() => openDialogOf("password")}>Change password…</button></li>
      <li><button type="button" role="menuitem" onclick={() => openDialogOf("settings")}>Settings…</button></li>
      <li class="sep"></li>
      <li><button type="button" role="menuitem" onclick={() => { open = false; vault.lock(); }}>Lock vault <kbd>⌘L</kbd></button></li>
    </ul>
  {/if}
</div>

{#if dialog === "password"}
  <ChangePassword onclose={() => (dialog = "none")} />
{:else if dialog === "settings"}
  <SettingsDialog onclose={() => (dialog = "none")} />
{:else if dialog === "export" && workspace.current}
  <ExportConfirm note={workspace.current} onclose={() => (dialog = "none")} />
{/if}

{#if toast}
  <div class="toast" role="status">{toast}</div>
{/if}

<style>
  .menu {
    position: relative;
  }
  .trigger {
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-muted);
    padding: 4px 10px;
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
  }
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 40;
  }
  ul {
    position: absolute;
    right: 0;
    top: calc(100% + 4px);
    z-index: 50;
    list-style: none;
    margin: 0;
    padding: 4px;
    min-width: 220px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.18);
  }
  li button {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    text-align: left;
    background: none;
    border: none;
    padding: 7px 10px;
    border-radius: 6px;
    cursor: pointer;
  }
  li button:hover:not(:disabled) {
    background: var(--bg-muted);
  }
  li button:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .sep {
    height: 1px;
    background: var(--border);
    margin: 4px 6px;
  }
  kbd {
    font-size: 11px;
    color: var(--fg-muted);
  }
  .toast {
    position: fixed;
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--fg);
    color: var(--bg);
    padding: 8px 14px;
    border-radius: 8px;
    font-size: 12px;
    max-width: 80vw;
    word-break: break-all;
    z-index: 200;
  }
</style>
