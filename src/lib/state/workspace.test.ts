import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { SAVE_DEBOUNCE_MS, workspace } from "./workspace.svelte";
import { callsTo, installFakeBackend, note, uninstallFakeBackend, type FakeBackend } from "../test-utils";

let backend: FakeBackend;

beforeEach(() => {
  vi.useFakeTimers();
  workspace.reset();
  backend = installFakeBackend({ notes: [note(1), note(2, { folder_id: 9 })] });
});

afterEach(() => {
  uninstallFakeBackend();
  vi.useRealTimers();
});

describe("workspace", () => {
  it("loads folders and every note when no folder is selected", async () => {
    await workspace.load();
    expect(workspace.notes.map((n) => n.id).sort()).toEqual([1, 2]);
    expect(callsTo(backend, "note_list_all")).toHaveLength(1);
    expect(callsTo(backend, "note_list")).toHaveLength(0);
  });

  it("lists only the selected folder's notes", async () => {
    await workspace.selectFolder(9);
    expect(workspace.notes.map((n) => n.id)).toEqual([2]);
    expect(callsTo(backend, "note_list")[0].args).toEqual({ folderId: 9 });
  });

  it("keeps the draft title untouched after an autosave normalises it", async () => {
    await workspace.load();
    await workspace.selectNote(1);
    workspace.updateDraft("Hello ", "body 1");
    await vi.advanceTimersByTimeAsync(SAVE_DEBOUNCE_MS);
    expect(backend.notes.get(1)?.title).toBe("Hello");
    expect(workspace.current?.title).toBe("Hello ");
    expect(workspace.notes.find((n) => n.id === 1)?.title).toBe("Hello");
    workspace.updateDraft("", "body 1");
    await vi.advanceTimersByTimeAsync(SAVE_DEBOUNCE_MS);
    expect(workspace.current?.title).toBe("");
    expect(backend.notes.get(1)?.title).toBe("Untitled");
  });

  it("coalesces rapid edits into one save after the debounce", async () => {
    await workspace.selectNote(1);
    workspace.updateDraft("Note 1", "a");
    workspace.updateDraft("Note 1", "ab");
    workspace.updateDraft("Note 1", "abc");
    expect(workspace.saveState).toBe("dirty");
    expect(callsTo(backend, "note_update")).toHaveLength(0);
    await vi.advanceTimersByTimeAsync(SAVE_DEBOUNCE_MS);
    const updates = callsTo(backend, "note_update");
    expect(updates).toHaveLength(1);
    expect(updates[0].args).toEqual({ id: 1, title: "Note 1", body: "abc" });
    expect(workspace.saveState).toBe("saved");
    expect(backend.notes.get(1)?.body).toBe("abc");
  });

  it("flushes a dirty draft before switching notes", async () => {
    await workspace.selectNote(1);
    workspace.updateDraft("Note 1", "unsaved text");
    await workspace.selectNote(2);
    const order = backend.calls.map((c) => c.cmd).filter((c) => c === "note_update" || c === "note_get");
    expect(order).toEqual(["note_get", "note_update", "note_get"]);
    expect(backend.notes.get(1)?.body).toBe("unsaved text");
    expect(workspace.current?.id).toBe(2);
  });

  it("does nothing when the draft equals the current note", async () => {
    await workspace.selectNote(1);
    workspace.updateDraft("Note 1", "body 1");
    await vi.advanceTimersByTimeAsync(SAVE_DEBOUNCE_MS * 2);
    expect(callsTo(backend, "note_update")).toHaveLength(0);
    expect(workspace.saveState).toBe("idle");
  });

  it("keeps the draft dirty and reports an error when saving fails", async () => {
    await workspace.selectNote(1);
    backend.failNext.note_update = { code: "db", message: "disk full" };
    workspace.updateDraft("Note 1", "x");
    await vi.advanceTimersByTimeAsync(SAVE_DEBOUNCE_MS);
    expect(workspace.saveState).toBe("error");
    expect(workspace.error).toBe("disk full");
    await workspace.flush();
    expect(workspace.saveState).toBe("saved");
    expect(backend.notes.get(1)?.body).toBe("x");
  });

  it("creates a note in the selected folder and selects it", async () => {
    await workspace.selectFolder(9);
    await workspace.createNote();
    expect(callsTo(backend, "note_create")[0].args).toEqual({ folderId: 9, title: "" });
    expect(workspace.current?.title).toBe("Untitled");
    expect(workspace.notes[0].id).toBe(workspace.selectedNoteId);
  });

  it("reset clears everything including a pending save", async () => {
    await workspace.selectNote(1);
    workspace.updateDraft("Note 1", "never saved");
    workspace.reset();
    await vi.advanceTimersByTimeAsync(SAVE_DEBOUNCE_MS * 2);
    expect(callsTo(backend, "note_update")).toHaveLength(0);
    expect(workspace.current).toBeNull();
    expect(workspace.notes).toEqual([]);
  });

  it("deleting the selected note clears the editor", async () => {
    await workspace.selectNote(1);
    await workspace.deleteNote(1);
    expect(workspace.current).toBeNull();
    expect(workspace.notes.find((n) => n.id === 1)).toBeUndefined();
  });

  it("surfaces locked errors instead of throwing", async () => {
    backend.locked = true;
    await workspace.load();
    expect(workspace.error).toBe("vault is locked");
  });
});
