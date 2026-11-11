<script lang="ts">
  import Editor from "../Editor.svelte";

  let body = $state("hello");

  export function getBody(): string {
    return body;
  }

  export function setBody(next: string): void {
    body = next;
  }
</script>

<Editor value={body} onchange={(b) => (body = b)} />
