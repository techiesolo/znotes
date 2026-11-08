// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import { renderMarkdown } from "./markdown";

describe("renderMarkdown", () => {
  it("renders GFM tables, code blocks and task lists", () => {
    const html = renderMarkdown("| a | b |\n|---|---|\n| 1 | 2 |\n\n```js\nlet x = 1;\n```\n\n- [x] done\n");
    expect(html).toContain("<table>");
    expect(html).toContain("<code");
    expect(html).toMatch(/<input[^>]*type="checkbox"/);
  });

  it("drops scripts and event handlers", () => {
    const html = renderMarkdown('<script>document.title="pwned"</script>\n\n<img src=x onerror="alert(1)">\n');
    expect(html).not.toContain("<script");
    expect(html).not.toContain("onerror");
    expect(html).not.toContain("pwned");
  });

  it("neutralises javascript: links and opens links in a new window", () => {
    const html = renderMarkdown("[x](javascript:alert(1)) [ok](https://example.com)");
    expect(html).not.toContain("javascript:");
    expect(html).toContain('href="https://example.com"');
    expect(html).toContain('rel="noopener noreferrer"');
  });

  it("keeps only disabled checkbox inputs", () => {
    const html = renderMarkdown('<input type="text" value="x"><input type="checkbox" checked>');
    expect(html).not.toContain('type="text"');
    expect(html).toMatch(/<input[^>]*type="checkbox"[^>]*>/);
    expect(html).toMatch(/<input[^>]*disabled/);
  });

  it("strips inline styles and iframes", () => {
    const html = renderMarkdown('<p style="position:fixed">x</p><iframe src="https://e.com"></iframe>');
    expect(html).not.toContain("style=");
    expect(html).not.toContain("<iframe");
    expect(html).toContain("<p>x</p>");
  });
});
