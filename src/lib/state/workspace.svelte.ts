import { api, errorMessage, type Folder, type Note, type NoteSummary } from "../api";
import { debounce } from "../utils/debounce";

export type SaveState = "idle" | "dirty" | "saving" | "saved" | "error";

export const SAVE_DEBOUNCE_MS = 500;

class WorkspaceState {
  folders = $state<Folder[]>([]);
  notes = $state<NoteSummary[]>([]);
  selectedFolderId = $state<number | null>(null);
  selectedNoteId = $state<number | null>(null);
  current = $state<Note | null>(null);
  saveState = $state<SaveState>("idle");
  error = $state<string | null>(null);
  preview = $state(false);

  private dirty = false;
  private saving: Promise<void> | null = null;
  private readonly scheduleFlush = debounce(() => void this.flush(), SAVE_DEBOUNCE_MS);

  async load(): Promise<void> {
    await this.guard(async () => {
      this.folders = await api.folderList();
      this.notes = await api.noteList(this.selectedFolderId);
    });
  }

  async refreshNotes(): Promise<void> {
    await this.guard(async () => {
      this.notes = await api.noteList(this.selectedFolderId);
    });
  }

  async selectFolder(id: number | null): Promise<void> {
    await this.flush();
    this.selectedFolderId = id;
    this.selectedNoteId = null;
    this.current = null;
    this.saveState = "idle";
    await this.refreshNotes();
  }

  async selectNote(id: number): Promise<void> {
    await this.flush();
    await this.guard(async () => {
      this.current = await api.noteGet(id);
      this.selectedNoteId = id;
      this.saveState = "idle";
    });
  }

  async createNote(): Promise<void> {
    await this.flush();
    await this.guard(async () => {
      const note = await api.noteCreate(this.selectedFolderId, "");
      this.notes = [summaryOf(note), ...this.notes];
      this.current = note;
      this.selectedNoteId = note.id;
      this.saveState = "idle";
    });
  }

  updateDraft(title: string, body: string): void {
    if (!this.current) return;
    if (this.current.title === title && this.current.body === body) return;
    this.current = { ...this.current, title, body };
    this.dirty = true;
    this.saveState = "dirty";
    this.scheduleFlush();
  }

  async flush(): Promise<void> {
    if (this.saving) await this.saving;
    if (!this.dirty || !this.current) return;
    this.scheduleFlush.cancel();
    const { id, title, body } = this.current;
    this.dirty = false;
    this.saveState = "saving";
    this.saving = (async () => {
      try {
        const saved = await api.noteUpdate(id, title, body);
        if (this.current?.id === id) {
          this.current = { ...this.current, title: saved.title, updated_at: saved.updated_at };
        }
        this.notes = this.notes
          .map((n) => (n.id === id ? summaryOf(saved) : n))
          .sort((a, b) => b.updated_at - a.updated_at || b.id - a.id);
        if (!this.dirty) this.saveState = "saved";
      } catch (err) {
        this.dirty = true;
        this.saveState = "error";
        this.error = errorMessage(err);
      } finally {
        this.saving = null;
      }
    })();
    await this.saving;
  }

  async deleteNote(id: number): Promise<void> {
    await this.guard(async () => {
      await api.noteDelete(id);
      this.notes = this.notes.filter((n) => n.id !== id);
      if (this.selectedNoteId === id) {
        this.selectedNoteId = null;
        this.current = null;
        this.dirty = false;
        this.scheduleFlush.cancel();
        this.saveState = "idle";
      }
    });
  }

  async createFolder(parentId: number | null, name: string): Promise<boolean> {
    return this.guard(async () => {
      const folder = await api.folderCreate(parentId, name);
      this.folders = [...this.folders, folder];
      return true;
    }).then((ok) => ok ?? false);
  }

  async renameFolder(id: number, name: string): Promise<boolean> {
    return this.guard(async () => {
      const folder = await api.folderRename(id, name);
      this.folders = this.folders.map((f) => (f.id === id ? folder : f));
      return true;
    }).then((ok) => ok ?? false);
  }

  async deleteFolder(id: number): Promise<void> {
    await this.flush();
    await this.guard(async () => {
      await api.folderDelete(id);
      const removed = new Set<number>();
      const collect = (parent: number) => {
        removed.add(parent);
        for (const f of this.folders) if (f.parent_id === parent) collect(f.id);
      };
      collect(id);
      this.folders = this.folders.filter((f) => !removed.has(f.id));
      if (this.selectedFolderId !== null && removed.has(this.selectedFolderId)) {
        await this.selectFolder(null);
      }
    });
  }

  reset(): void {
    this.scheduleFlush.cancel();
    this.dirty = false;
    this.saving = null;
    this.folders = [];
    this.notes = [];
    this.selectedFolderId = null;
    this.selectedNoteId = null;
    this.current = null;
    this.saveState = "idle";
    this.error = null;
  }

  togglePreview(): void {
    this.preview = !this.preview;
  }

  private async guard<T>(action: () => Promise<T>): Promise<T | undefined> {
    try {
      this.error = null;
      return await action();
    } catch (err) {
      this.error = errorMessage(err);
      return undefined;
    }
  }
}

function summaryOf(note: Note): NoteSummary {
  return { id: note.id, folder_id: note.folder_id, title: note.title, updated_at: note.updated_at };
}

export const workspace = new WorkspaceState();
