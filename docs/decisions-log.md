# Decisions log (autonomous run)

Choices made without asking during the `zk:cook` run on 2026-11-08, for the owner to check.
Each entry: what was decided, why, and how to reverse it if you disagree.

| # | Decision | Why | Reverse by |
|---|---|---|---|
| 1 | Plain Vite + Svelte 5 (no SvelteKit) | Desktop SPA needs no routing/SSR; fewer deps | Re-scaffold with `create-tauri-app` svelte template |
| 2 | Scaffold files written by hand instead of `pnpm create vite` | create-vite is interactive in a non-empty dir | n/a (same output) |
| 3 | Commit plans/ and docs/ to main | user asked to work on main, push after each step | n/a |
