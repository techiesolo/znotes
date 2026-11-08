import { api, errorMessage, isAppError, onVaultLocked } from "../api";
import { workspace } from "./workspace.svelte";

export type VaultUiStatus = "loading" | "locked" | "open";

const FRIENDLY: Record<string, string> = {
  wrong_password: "Wrong password",
  not_a_vault: "No vault found in that folder",
  already_exists: "A vault already exists in that folder",
  invalid_input: "Password must not be empty",
  unsupported_version: "This vault was created by a newer version of znotes",
};

class VaultState {
  status = $state<VaultUiStatus>("loading");
  dir = $state<string | null>(null);
  error = $state<string | null>(null);
  busy = $state(false);
  lockReason = $state<"idle" | "manual" | null>(null);

  async refresh(): Promise<void> {
    const status = await api.vaultStatus();
    this.dir = status.dir;
    if (status.locked) {
      this.markLocked(null);
    } else {
      this.status = "open";
      await workspace.load();
    }
  }

  async open(dir: string, password: string): Promise<boolean> {
    return this.unlockWith(() => api.vaultOpen(dir, password));
  }

  async create(dir: string, password: string): Promise<boolean> {
    return this.unlockWith(() => api.vaultCreate(dir, password));
  }

  async lock(): Promise<void> {
    await workspace.flush();
    await api.vaultLock();
    this.markLocked("manual");
  }

  async listenForLock(): Promise<() => void> {
    return onVaultLocked((payload) => this.markLocked(payload.reason));
  }

  markLocked(reason: "idle" | "manual" | null): void {
    workspace.reset();
    this.status = "locked";
    this.lockReason = reason;
  }

  private async unlockWith(action: () => Promise<{ dir: string }>): Promise<boolean> {
    this.busy = true;
    this.error = null;
    try {
      const info = await action();
      this.dir = info.dir;
      this.lockReason = null;
      this.status = "open";
      await workspace.load();
      return true;
    } catch (err) {
      this.error = isAppError(err) ? (FRIENDLY[err.code] ?? err.message) : errorMessage(err);
      return false;
    } finally {
      this.busy = false;
    }
  }
}

export const vault = new VaultState();
