import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import ChangePassword from "./ChangePassword.svelte";
import { installFakeBackend, uninstallFakeBackend, type FakeBackend } from "../test-utils";

let backend: FakeBackend;

beforeEach(() => {
  backend = installFakeBackend();
});

afterEach(uninstallFakeBackend);

async function fill(current: string, next: string, confirm: string) {
  await fireEvent.input(screen.getByLabelText("Current password"), { target: { value: current } });
  await fireEvent.input(screen.getByLabelText("New password"), { target: { value: next } });
  await fireEvent.input(screen.getByLabelText("Confirm new password"), { target: { value: confirm } });
}

describe("ChangePassword", () => {
  it("keeps the submit disabled until the new passwords match", async () => {
    render(ChangePassword, { onclose: vi.fn() });
    await fill("pw", "new", "nope");
    expect(screen.getByRole("button", { name: "Change password" })).toBeDisabled();
    await fireEvent.input(screen.getByLabelText("Confirm new password"), { target: { value: "new" } });
    expect(screen.getByRole("button", { name: "Change password" })).toBeEnabled();
  });

  it("reports a wrong current password and clears that field", async () => {
    render(ChangePassword, { onclose: vi.fn() });
    await fill("wrong", "new", "new");
    await fireEvent.click(screen.getByRole("button", { name: "Change password" }));
    await waitFor(() => expect(screen.getByRole("alert")).toHaveTextContent("Current password is wrong"));
    expect(screen.getByLabelText("Current password")).toHaveValue("");
    expect(backend.password).toBe("pw");
  });

  it("changes the password and shows a confirmation", async () => {
    const onclose = vi.fn();
    render(ChangePassword, { onclose });
    await fill("pw", "new", "new");
    await fireEvent.click(screen.getByRole("button", { name: "Change password" }));
    await waitFor(() => expect(screen.getByText(/Password changed/)).toBeInTheDocument());
    expect(backend.password).toBe("new");
    await fireEvent.click(screen.getByRole("button", { name: "Done" }));
    expect(onclose).toHaveBeenCalled();
  });
});
