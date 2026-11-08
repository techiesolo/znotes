import { describe, expect, it } from "vitest";
import { formatRelative } from "./time";

describe("formatRelative", () => {
  const now = 1_700_000_000;
  it("formats recent timestamps by unit", () => {
    expect(formatRelative(now, now)).toBe("just now");
    expect(formatRelative(now - 4, now)).toBe("just now");
    expect(formatRelative(now - 45, now)).toBe("45s ago");
    expect(formatRelative(now - 60 * 7, now)).toBe("7m ago");
    expect(formatRelative(now - 3600 * 5, now)).toBe("5h ago");
    expect(formatRelative(now - 86400 * 3, now)).toBe("3d ago");
  });
  it("never reports negative durations", () => {
    expect(formatRelative(now + 500, now)).toBe("just now");
  });
  it("falls back to a date after a month", () => {
    const out = formatRelative(now - 86400 * 40, now);
    expect(out).not.toMatch(/ago$/);
    expect(out.length).toBeGreaterThan(0);
  });
});
