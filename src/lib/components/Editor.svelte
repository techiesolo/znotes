<script lang="ts">
  import { onMount } from "svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView, keymap, placeholder as cmPlaceholder } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
  import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
  import { syntaxHighlighting, defaultHighlightStyle } from "@codemirror/language";

  interface Props {
    value: string;
    onchange?: (value: string) => void;
    placeholder?: string;
  }

  let { value, onchange, placeholder = "Start writing…" }: Props = $props();

  let host: HTMLDivElement;
  let view: EditorView | undefined;

  const theme = EditorView.theme({
    "&": { height: "100%", backgroundColor: "var(--bg)", color: "var(--fg)" },
    ".cm-scroller": {
      fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace",
      fontSize: "14px",
      lineHeight: "1.6",
      padding: "12px 16px",
    },
    ".cm-content": { caretColor: "var(--fg)" },
    "&.cm-focused": { outline: "none" },
    ".cm-gutters": { display: "none" },
    ".cm-placeholder": { color: "var(--fg-muted)" },
    ".cm-activeLine": { backgroundColor: "transparent" },
  });

  onMount(() => {
    view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: value,
        extensions: [
          history(),
          keymap.of([...defaultKeymap, ...historyKeymap, indentWithTab]),
          markdown({ base: markdownLanguage }),
          syntaxHighlighting(defaultHighlightStyle),
          EditorView.lineWrapping,
          cmPlaceholder(placeholder),
          theme,
          EditorView.updateListener.of((update) => {
            if (update.docChanged) onchange?.(update.state.doc.toString());
          }),
        ],
      }),
    });
    return () => {
      view?.destroy();
      view = undefined;
    };
  });

  $effect(() => {
    const next = value;
    if (!view) return;
    const current = view.state.doc.toString();
    if (current !== next) {
      view.dispatch({ changes: { from: 0, to: current.length, insert: next } });
    }
  });

  export function focus(): void {
    view?.focus();
  }
</script>

<div class="editor" bind:this={host}></div>

<style>
  .editor {
    height: 100%;
    min-height: 0;
    overflow: hidden;
  }
</style>
