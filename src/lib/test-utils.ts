import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import type { Folder, Note, NoteSummary, SearchHit, Settings } from "./api";

export interface Call {
  cmd: string;
  args: Record<string, unknown>;
}

export interface FakeBackend {
  calls: Call[];
  notes: Map<number, Note>;
  folders: Map<number, Folder>;
  locked: boolean;
  searchHits: SearchHit[];
  settings: Settings;
  exports: { id: number; destPath: string }[];
  backups: string[];
  password: string;
  failNext: Partial<Record<string, { code: string; message: string }>>;
}

export function note(id: number, overrides: Partial<Note> = {}): Note {
  return {
    id,
    folder_id: null,
    title: `Note ${id}`,
    body: `body ${id}`,
    created_at: 1000 + id,
    updated_at: 1000 + id,
    ...overrides,
  };
}

export function folder(id: number, overrides: Partial<Folder> = {}): Folder {
  return { id, parent_id: null, name: `Folder ${id}`, created_at: 1, updated_at: 1, ...overrides };
}

function summary(n: Note): NoteSummary {
  return { id: n.id, folder_id: n.folder_id, title: n.title, updated_at: n.updated_at };
}

export function installFakeBackend(seed: { notes?: Note[]; folders?: Folder[] } = {}): FakeBackend {
  const backend: FakeBackend = {
    calls: [],
    notes: new Map((seed.notes ?? []).map((n) => [n.id, n])),
    folders: new Map((seed.folders ?? []).map((f) => [f.id, f])),
    locked: false,
    searchHits: [],
    settings: { autolock_minutes: 10, last_vault_dir: null, preview: false },
    exports: [],
    backups: [],
    password: "pw",
    failNext: {},
  };
  let nextId = 1000;
  let clock = 5000;

  mockIPC((cmd, payload) => {
    const args = (payload ?? {}) as Record<string, unknown>;
    backend.calls.push({ cmd, args });
    const failure = backend.failNext[cmd];
    if (failure) {
      delete backend.failNext[cmd];
      throw failure;
    }
    if (cmd.startsWith("plugin:event|")) return 1;
    if (cmd === "plugin:dialog|open") return "/picked/dir";
    if (cmd === "plugin:dialog|save") return "/picked/export.md";
    if (cmd === "settings_get") return backend.settings;
    if (cmd === "settings_set") {
      const next = args.settings as Settings;
      if (next.autolock_minutes < 1 || next.autolock_minutes > 120) {
        throw { code: "invalid_input", message: "autolock must be between 1 and 120 minutes" };
      }
      backend.settings = next;
      return next;
    }
    if (cmd === "vault_status") return { locked: backend.locked, dir: backend.locked ? null : "/v" };
    if (cmd === "vault_open" || cmd === "vault_create") {
      backend.locked = false;
      return { dir: args.dir };
    }
    if (cmd === "vault_lock") {
      backend.locked = true;
      return null;
    }
    if (backend.locked) throw { code: "locked", message: "vault is locked" };
    switch (cmd) {
      case "vault_backup":
        backend.backups.push(args.destDir as string);
        return { db: `${args.destDir}/znotes-backup-x.znotes`, meta: `${args.destDir}/znotes-backup-x.meta.json` };
      case "vault_change_password":
        if (args.current !== backend.password) throw { code: "wrong_password", message: "wrong password" };
        backend.password = args.new as string;
        return null;
      case "note_export_md":
        backend.exports.push({ id: args.id as number, destPath: args.destPath as string });
        return null;
      case "folder_list":
        return [...backend.folders.values()];
      case "folder_create": {
        const f = folder(++nextId, { parent_id: args.parentId as number | null, name: args.name as string });
        backend.folders.set(f.id, f);
        return f;
      }
      case "folder_rename": {
        const f = backend.folders.get(args.id as number)!;
        const renamed = { ...f, name: args.name as string };
        backend.folders.set(f.id, renamed);
        return renamed;
      }
      case "folder_delete":
        backend.folders.delete(args.id as number);
        return null;
      case "note_list":
        return [...backend.notes.values()]
          .filter((n) => n.folder_id === (args.folderId ?? null))
          .map(summary)
          .sort((a, b) => b.updated_at - a.updated_at);
      case "note_get": {
        const n = backend.notes.get(args.id as number);
        if (!n) throw { code: "not_found", message: `note ${args.id} not found` };
        return n;
      }
      case "note_create": {
        const n = note(++nextId, {
          folder_id: (args.folderId as number | null) ?? null,
          title: (args.title as string) || "Untitled",
          body: "",
          updated_at: ++clock,
          created_at: clock,
        });
        backend.notes.set(n.id, n);
        return n;
      }
      case "note_update": {
        const n = backend.notes.get(args.id as number);
        if (!n) throw { code: "not_found", message: `note ${args.id} not found` };
        const updated = {
          ...n,
          title: (args.title as string).trim() || "Untitled",
          body: args.body as string,
          updated_at: ++clock,
        };
        backend.notes.set(n.id, updated);
        return updated;
      }
      case "note_move": {
        const n = backend.notes.get(args.id as number);
        if (!n) throw { code: "not_found", message: `note ${args.id} not found` };
        const moved = { ...n, folder_id: (args.folderId as number | null) ?? null, updated_at: ++clock };
        backend.notes.set(n.id, moved);
        return moved;
      }
      case "note_delete":
        backend.notes.delete(args.id as number);
        return null;
      case "search":
        return backend.searchHits;
      default:
        throw new Error(`unmocked command ${cmd}`);
    }
  });
  return backend;
}

export function uninstallFakeBackend(): void {
  clearMocks();
}

export function callsTo(backend: FakeBackend, cmd: string): Call[] {
  return backend.calls.filter((c) => c.cmd === cmd);
}

export async function flushMicrotasks(): Promise<void> {
  for (let i = 0; i < 5; i++) await Promise.resolve();
}
