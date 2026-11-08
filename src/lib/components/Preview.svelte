<script lang="ts">
  import { renderMarkdown } from "../markdown";

  interface Props {
    source: string;
  }

  let { source }: Props = $props();
  const html = $derived(renderMarkdown(source));
</script>

<article class="preview">
  {@html html}
</article>

<style>
  .preview {
    height: 100%;
    overflow-y: auto;
    padding: 12px 20px;
    line-height: 1.6;
    border-left: 1px solid var(--border);
  }
  .preview :global(h1),
  .preview :global(h2),
  .preview :global(h3) {
    margin: 1.2em 0 0.5em;
    line-height: 1.25;
  }
  .preview :global(pre) {
    background: var(--bg-muted);
    padding: 10px 12px;
    border-radius: 6px;
    overflow-x: auto;
  }
  .preview :global(code) {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 13px;
  }
  .preview :global(table) {
    border-collapse: collapse;
  }
  .preview :global(td),
  .preview :global(th) {
    border: 1px solid var(--border);
    padding: 4px 8px;
  }
  .preview :global(blockquote) {
    margin: 0;
    padding-left: 12px;
    border-left: 3px solid var(--border);
    color: var(--fg-muted);
  }
  .preview :global(a) {
    color: var(--accent);
  }
  .preview :global(img) {
    max-width: 100%;
  }
</style>
