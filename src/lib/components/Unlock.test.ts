import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import Unlock from "./Unlock.svelte";
import { vault } from "../state/vault.svelte";
import { workspace } from "../state/workspace.svelte";
import { callsTo, installFakeBackend, uninstallFakeBackend, type FakeBackend } from "../test-utils";

let backend: FakeBackend;

beforeEach(() => {
  workspace.reset();
  vault.status = "locked";
  vault.error = null;
  vault.dir = null;
  backend = installFakeBackend();
  backend.locked = true;
});

afterEach(uninstallFakeBackend);

async function fill(dir: string, password: string) {
  await fireEvent.input(screen.getByPlaceholderText("/path/to/vault"), { target: { value: dir } });
  await fireEvent.input(screen.getByLabelText("Password"), { target: { value: password } });
}

describe("Unlock", () => {
  it("shows a friendly message and clears the password on a wrong password", async () => {
    render(Unlock);
    backend.failNext.vault_open = { code: "wrong_password", message: "wrong password" };
    await fill("/v", "nope");
    await fireEvent.click(screen.getByRole("button", { name: "Unlock" }));
    await waitFor(() => expect(screen.getByRole("alert")).toHaveTextContent("Wrong password"));
    expect(screen.getByLabelText("Password")).toHaveValue("");
    expect(vault.status).toBe("locked");
  });

  it("opens the vault and loads the workspace on success", async () => {
    render(Unlock);
    await fill("/v", "pw");
    await fireEvent.click(screen.getByRole("button", { name: "Unlock" }));
    await waitFor(() => expect(vault.status).toBe("open"));
    expect(callsTo(backend, "vault_open")[0].args).toEqual({ dir: "/v", password: "pw" });
    expect(callsTo(backend, "folder_list")).toHaveLength(1);
  });

  it("requires matching passwords when creating", async () => {
    render(Unlock);
    await fireEvent.click(screen.getByRole("tab", { name: "Create vault" }));
    await fill("/new", "pw");
    await fireEvent.input(screen.getByLabelText("Confirm password"), { target: { value: "pw2" } });
    expect(screen.getByRole("button", { name: "Create and unlock" })).toBeDisabled();
    await fireEvent.input(screen.getByLabelText("Confirm password"), { target: { value: "pw" } });
    expect(screen.getByRole("button", { name: "Create and unlock" })).toBeEnabled();
    await fireEvent.click(screen.getByRole("button", { name: "Create and unlock" }));
    await waitFor(() => expect(vault.status).toBe("open"));
    expect(callsTo(backend, "vault_create")[0].args).toEqual({ dir: "/new", password: "pw" });
  });

  it("fills the folder from the directory picker", async () => {
    render(Unlock);
    await fireEvent.click(screen.getByRole("button", { name: "Choose…" }));
    await waitFor(() => expect(screen.getByPlaceholderText("/path/to/vault")).toHaveValue("/picked/dir"));
  });
});
