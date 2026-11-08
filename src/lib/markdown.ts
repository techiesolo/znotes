import DOMPurify from "dompurify";
import { marked } from "marked";

marked.setOptions({ gfm: true, breaks: false });

const purifier = DOMPurify();
purifier.addHook("afterSanitizeAttributes", (node) => {
  if (node.tagName === "A") {
    node.setAttribute("target", "_blank");
    node.setAttribute("rel", "noopener noreferrer");
  }
  if (node.tagName === "INPUT") {
    if (node.getAttribute("type") !== "checkbox") {
      node.remove();
    } else {
      node.setAttribute("disabled", "");
    }
  }
});

export function renderMarkdown(source: string): string {
  const html = marked.parse(source, { async: false });
  return purifier.sanitize(html, {
    USE_PROFILES: { html: true },
    FORBID_TAGS: ["style", "form", "button", "iframe", "object", "embed", "textarea", "select"],
    FORBID_ATTR: ["style"],
  });
}
