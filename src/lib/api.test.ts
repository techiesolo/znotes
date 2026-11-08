import { afterEach, describe, expect, it } from "vitest";
import { api, isAppError } from "./api";
import { callsTo, installFakeBackend, note, uninstallFakeBackend } from "./test-utils";

afterEach(uninstallFakeBackend);

describe("api", () => {
  it("passes arguments under the camelCase names the Rust commands expect", async () => {
    const backend = installFakeBackend({ notes: [note(7)] });
    await api.noteCreate(3, "x");
    await api.noteMove(7, null);
    await api.folderCreate(null, "Work");
    await api.search("q", 10);
    expect(callsTo(backend, "note_create")[0].args).toEqual({ folderId: 3, title: "x" });
    expect(callsTo(backend, "note_move")[0].args).toEqual({ id: 7, folderId: null });
    expect(callsTo(backend, "folder_create")[0].args).toEqual({ parentId: null, name: "Work" });
    expect(callsTo(backend, "search")[0].args).toEqual({ query: "q", limit: 10 });
  });

  it("defaults the search limit to 50", async () => {
    const backend = installFakeBackend();
    await api.search("q");
    expect(callsTo(backend, "search")[0].args.limit).toBe(50);
  });

  it("surfaces backend errors as AppError objects", async () => {
    const backend = installFakeBackend();
    backend.locked = true;
    await expect(api.folderList()).rejects.toSatisfy((err: unknown) => isAppError(err) && err.code === "locked");
  });

  it("isAppError rejects non-conforming values", () => {
    expect(isAppError(null)).toBe(false);
    expect(isAppError("locked")).toBe(false);
    expect(isAppError({ code: 1, message: "x" })).toBe(false);
    expect(isAppError({ code: "x", message: "y" })).toBe(true);
  });
});
