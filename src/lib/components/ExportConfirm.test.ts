import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import ExportConfirm from "./ExportConfirm.svelte";
import { installFakeBackend, note, uninstallFakeBackend, type FakeBackend } from "../test-utils";

let backend: FakeBackend;

beforeEach(() => {
  backend = installFakeBackend({ notes: [note(1, { title: "Secrets/2026" })] });
});

afterEach(uninstallFakeBackend);

describe("ExportConfirm", () => {
  it("warns before exporting and only writes after the explicit action", async () => {
    render(ExportConfirm, { note: note(1, { title: "Secrets/2026" }), onclose: vi.fn() });
    expect(screen.getByText(/unencrypted file/)).toBeInTheDocument();
    expect(backend.exports).toHaveLength(0);
    await fireEvent.click(screen.getByRole("button", { name: "Choose location and export" }));
    await waitFor(() => expect(backend.exports).toEqual([{ id: 1, destPath: "/picked/export.md" }]));
    expect(await screen.findByText(/Exported to/)).toBeInTheDocument();
    const saveCall = backend.calls.find((c) => c.cmd === "plugin:dialog|save");
    expect((saveCall?.args.options as { defaultPath?: string })?.defaultPath).toBe("Secrets_2026.md");
  });

  it("cancel closes without exporting", async () => {
    const onclose = vi.fn();
    render(ExportConfirm, { note: note(1), onclose });
    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onclose).toHaveBeenCalled();
    expect(backend.exports).toHaveLength(0);
  });
});
