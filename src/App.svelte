<script lang="ts">
  import Editor from "./lib/components/Editor.svelte";

  let text = $state("# znotes\n\nSpike: Tauri v2 + Svelte 5 + CodeMirror 6.\n\n- [ ] renders on macOS\n- [ ] renders on Ubuntu\n");
</script>

<main>
  <Editor bind:value={text} />
</main>

<style>
  main {
    height: 100%;
    padding: 16px;
  }
</style>
