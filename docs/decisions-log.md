# Decisions log (autonomous run)

Choices made without asking during the `zk:cook` run on 2026-11-08, for the owner to check.
Each entry: what was decided, why, and how to reverse it if you disagree.

| # | Decision | Why | Reverse by |
|---|---|---|---|
| 1 | Plain Vite + Svelte 5 (no SvelteKit) | Desktop SPA needs no routing/SSR; fewer deps | Re-scaffold with `create-tauri-app` svelte template |
| 2 | Scaffold files written by hand instead of `pnpm create vite` | create-vite is interactive in a non-empty dir | n/a (same output) |
| 3 | Commit plans/ and docs/ to main | user asked to work on main, push after each step | n/a |
| 4 | Title weight 10× body in search ranking (`bm25(notes_fts, 10.0, 1.0)`) | A note titled "rust" should outrank a note whose body merely repeats "rust" | Change the weights in `crates/znotes-core/src/repo/search.rs` |
| 5 | Autolock watchdog is a std thread ticking every 15 s, state stored in ms | No tokio dependency needed; 15 s granularity is fine for a 1–120 min setting | `src-tauri/src/watchdog.rs` |
| 6 | IPC tests use `tauri::test` mock runtime with `tauri://localhost` origin on macOS/Linux | The docs example uses `http://tauri.localhost`, which is the Windows origin and fails ACL on macOS | `src-tauri/tests/ipc.rs::local_origin` |
| 7 | Phase 03 manual devtools check replaced by an automated IPC test | Autonomous run cannot read devtools; the mock runtime exercises the same command layer | Run the devtools check yourself if you want to see it live |
| 8 | Frontend tests run on `happy-dom` + vitest 4.1 instead of jsdom + vitest 5 | jsdom 31 (undici 8) and vitest 5 both require Node 22; this machine has Node 20.19 | Upgrade Node to 22+, then `pnpm add -D vitest@5 jsdom` and set `environment: "jsdom"` |
| 9 | Manual UI criteria of Phase 04 (create vault → type → relaunch → search; ⌘L; empty web storage) are left for the owner | The autonomous run has no display/devtools access; logic is covered by 24 vitest + 63 cargo tests | Run `pnpm tauri dev` and walk the checklist in the phase file |
| 10 | Editor undo history resets when switching notes (`{#key}` remount) | Simplest correct behaviour; per-note EditorState cache deferred | Cache `EditorState` per note id in `Workspace.svelte` |
