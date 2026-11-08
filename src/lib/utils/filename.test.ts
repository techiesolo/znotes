import { describe, expect, it } from "vitest";
import { suggestedExportName } from "./filename";

describe("suggestedExportName", () => {
  it("replaces path and shell characters", () => {
    expect(suggestedExportName("Hello/World: v2")).toBe("Hello_World_ v2.md");
    expect(suggestedExportName("../../etc/passwd")).toBe("______etc_passwd.md");
  });
  it("keeps unicode letters", () => {
    expect(suggestedExportName("tiếng việt")).toBe("tiếng việt.md");
  });
  it("falls back for empty titles", () => {
    expect(suggestedExportName("   ")).toBe("note.md");
  });
});
