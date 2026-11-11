import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import Search from "./Search.svelte";
import { workspace } from "../state/workspace.svelte";
import { callsTo, installFakeBackend, note, uninstallFakeBackend, type FakeBackend } from "../test-utils";

let backend: FakeBackend;

beforeEach(() => {
  vi.useFakeTimers();
  workspace.reset();
  backend = installFakeBackend({ notes: [note(1, { body: "tiếng việt" })] });
  backend.searchHits = [{ id: 1, folder_id: null, title: "Note 1", snippet: "…\u0002tiếng\u0003 việt…" }];
});

afterEach(() => {
  uninstallFakeBackend();
  vi.useRealTimers();
});

describe("Search", () => {
  it("debounces input, renders hits with highlighted snippets, and opens the note on click", async () => {
    render(Search);
    const input = screen.getByLabelText("Search notes");
    await fireEvent.input(input, { target: { value: "tieng" } });
    expect(callsTo(backend, "search")).toHaveLength(0);
    await vi.advanceTimersByTimeAsync(200);
    expect(callsTo(backend, "search")).toHaveLength(1);
    expect(callsTo(backend, "search")[0].args).toEqual({ query: "tieng", limit: 50 });

    const option = await screen.findByRole("option");
    expect(option).toHaveTextContent("Note 1");
    expect(option.querySelector("mark")).toHaveTextContent("tiếng");
    expect(option.textContent).not.toContain("\u0002");

    await fireEvent.click(option);
    await vi.advanceTimersByTimeAsync(0);
    expect(workspace.selectedNoteId).toBe(1);
    expect(workspace.current?.body).toBe("tiếng việt");
  });

  it("shows no results for an empty hit list", async () => {
    backend.searchHits = [];
    render(Search);
    await fireEvent.input(screen.getByLabelText("Search notes"), { target: { value: "zzz" } });
    await vi.advanceTimersByTimeAsync(200);
    expect(await screen.findByText("No matches")).toBeInTheDocument();
  });
});
