import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface VaultStatus {
  locked: boolean;
  dir: string | null;
}

export interface VaultInfo {
  dir: string;
}

export interface Folder {
  id: number;
  parent_id: number | null;
  name: string;
  created_at: number;
  updated_at: number;
}

export interface NoteSummary {
  id: number;
  folder_id: number | null;
  title: string;
  updated_at: number;
}

export interface Note extends NoteSummary {
  body: string;
  created_at: number;
}

export interface SearchHit {
  id: number;
  folder_id: number | null;
  title: string;
  snippet: string;
}

export interface Settings {
  autolock_minutes: number;
  last_vault_dir: string | null;
  preview: boolean;
}

export interface BackupPaths {
  db: string;
  meta: string;
}

export interface AppError {
  code: string;
  message: string;
}

export function isAppError(value: unknown): value is AppError {
  return (
    typeof value === "object" &&
    value !== null &&
    typeof (value as AppError).code === "string" &&
    typeof (value as AppError).message === "string"
  );
}

export function errorMessage(value: unknown): string {
  if (isAppError(value)) return value.message;
  if (value instanceof Error) return value.message;
  return String(value);
}

export const api = {
  vaultStatus: () => invoke<VaultStatus>("vault_status"),
  vaultOpen: (dir: string, password: string) => invoke<VaultInfo>("vault_open", { dir, password }),
  vaultCreate: (dir: string, password: string) => invoke<VaultInfo>("vault_create", { dir, password }),
  vaultLock: () => invoke<void>("vault_lock"),

  folderList: () => invoke<Folder[]>("folder_list"),
  folderCreate: (parentId: number | null, name: string) =>
    invoke<Folder>("folder_create", { parentId, name }),
  folderRename: (id: number, name: string) => invoke<Folder>("folder_rename", { id, name }),
  folderDelete: (id: number) => invoke<void>("folder_delete", { id }),

  noteList: (folderId: number | null) => invoke<NoteSummary[]>("note_list", { folderId }),
  noteListAll: () => invoke<NoteSummary[]>("note_list_all"),
  noteGet: (id: number) => invoke<Note>("note_get", { id }),
  noteCreate: (folderId: number | null, title: string) =>
    invoke<Note>("note_create", { folderId, title }),
  noteUpdate: (id: number, title: string, body: string) =>
    invoke<Note>("note_update", { id, title, body }),
  noteMove: (id: number, folderId: number | null) => invoke<Note>("note_move", { id, folderId }),
  noteDelete: (id: number) => invoke<void>("note_delete", { id }),

  search: (query: string, limit = 50) => invoke<SearchHit[]>("search", { query, limit }),

  vaultBackup: (destDir: string) => invoke<BackupPaths>("vault_backup", { destDir }),
  vaultChangePassword: (current: string, next: string) =>
    invoke<void>("vault_change_password", { current, new: next }),
  noteExportMd: (id: number, destPath: string) => invoke<void>("note_export_md", { id, destPath }),

  settingsGet: () => invoke<Settings>("settings_get"),
  settingsSet: (settings: Settings) => invoke<Settings>("settings_set", { settings }),
};

export interface LockedPayload {
  reason: "idle" | "manual";
}

export function onVaultLocked(handler: (payload: LockedPayload) => void): Promise<UnlistenFn> {
  return listen<LockedPayload>("vault-locked", (event) => handler(event.payload));
}
