import { describe, expect, it } from "vitest";
import { render } from "@testing-library/svelte";
import { flushSync, tick } from "svelte";
import { EditorView } from "@codemirror/view";
import EditorHost from "./harness/EditorHost.svelte";

function viewIn(container: HTMLElement): EditorView {
  const el = container.querySelector(".cm-editor") as HTMLElement;
  return EditorView.findFromDOM(el)!;
}

describe("Editor", () => {
  it("keeps the same EditorView, cursor and undo history while typing", async () => {
    const { container, component } = render(EditorHost);
    const before = container.querySelector(".cm-editor");
    const view = viewIn(container);
    view.dispatch({ changes: { from: 5, insert: "!" }, selection: { anchor: 6 } });
    await tick();
    flushSync();
    view.dispatch({ changes: { from: 6, insert: "?" }, selection: { anchor: 7 } });
    await tick();
    flushSync();

    expect(container.querySelectorAll(".cm-editor")).toHaveLength(1);
    expect(container.querySelector(".cm-editor")).toBe(before);
    const after = viewIn(container);
    expect(after.state.doc.toString()).toBe("hello!?");
    expect(after.state.selection.main.head).toBe(7);
    expect(component.getBody()).toBe("hello!?");
  });

  it("replaces the document when the value changes from outside", async () => {
    const { container, component } = render(EditorHost);
    component.setBody("# replaced");
    await tick();
    flushSync();
    expect(viewIn(container).state.doc.toString()).toBe("# replaced");
    expect(container.querySelectorAll(".cm-editor")).toHaveLength(1);
  });
});
