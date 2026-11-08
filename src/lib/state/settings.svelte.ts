import { api, errorMessage, type Settings } from "../api";

const DEFAULTS: Settings = { autolock_minutes: 10, last_vault_dir: null, preview: false };

class SettingsState {
  current = $state<Settings>({ ...DEFAULTS });
  error = $state<string | null>(null);
  loaded = $state(false);

  async load(): Promise<void> {
    try {
      this.current = await api.settingsGet();
      this.error = null;
    } catch (err) {
      this.error = errorMessage(err);
    } finally {
      this.loaded = true;
    }
  }

  async save(patch: Partial<Settings>): Promise<boolean> {
    const next = { ...this.current, ...patch };
    try {
      this.current = await api.settingsSet(next);
      this.error = null;
      return true;
    } catch (err) {
      this.error = errorMessage(err);
      return false;
    }
  }

  reset(): void {
    this.current = { ...DEFAULTS };
    this.error = null;
    this.loaded = false;
  }
}

export const settings = new SettingsState();
