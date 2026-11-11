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
| 11 | Markdown preview sanitizer tests run under jsdom 26 (per-file `@vitest-environment`), other tests under happy-dom | DOMPurify behaves incorrectly on happy-dom 21 (kept `onerror`, dropped `<table>`), so a happy-dom test would prove nothing; jsdom 26 is the newest jsdom that runs on Node 20 | Move everything to jsdom once Node ≥ 22 is available |
| 12 | Preview keeps task-list checkboxes but forces them `disabled`; all other inputs/forms/iframes removed | Task lists are core GFM; a disabled checkbox cannot submit anything | `src/lib/markdown.ts` hook |
| 13 | Export writes only the note body (no title header); filename suggested from title with `/`, `:` etc. replaced by `_` | Body is the markdown the user wrote; a synthesized `# title` line would not round-trip | `ExportConfirm.svelte` / `commands/vault.rs::note_export_md` |
| 14 | Backup file name `znotes-backup-YYYYMMDD-HHmm.znotes` computed in UTC without a date crate | Avoids a `chrono`/`time` dependency for one timestamp | `commands/vault.rs::timestamp` |
| 15 | Settings file lives in Tauri's `app_config_dir` (`~/Library/Application Support/com.techiesolo.znotes` on macOS, `~/.config/com.techiesolo.znotes` on Linux) and holds only autolock minutes, last vault dir, preview flag | Plaintext by design; nothing sensitive | n/a |
| 16 | Argon2id default raised to m=128 MiB, t=4, p=1 (from 64 MiB/t=3) | Release-mode measurement on this Mac: 64 MiB/t=3 = 98 ms, far below the 0.5–1 s target; 128 MiB/t=4 = 278 ms here, expected ~0.6–1 s on a slower laptop | `KdfParams::default()`; existing vaults keep their own params |
| 17 | Webview window set to `incognito: true` | WebKit then uses a non-persistent data store, so page cache/localStorage/IndexedDB never touch disk; defence in depth for the leak audit | `src-tauri/tauri.conf.json` |
| 18 | App icon generated procedurally (blue rounded square, page with lock) via a stdlib Python script, then `pnpm tauri icon`; mobile icon sets deleted | No designer input available; placeholder that is clearly not the Tauri default | Replace `src-tauri/icons/*` with `pnpm tauri icon <your.png>` |
| 19 | Release pipeline uses `tauri-apps/tauri-action` creating a **draft** GitHub release on `v*` tags, unsigned, with SHA-256 sums in the job log | Signing/notarization out of scope for v1 | `.github/workflows/release.yml` |
| 20 | Leak audit on macOS performed only for the backend path (fixture + script); full UI protocol and the Ubuntu run are left to the owner | No display/devtools in the autonomous session | `docs/leak-audit.md` results table |
| 21 | `default_params_open_time_is_reasonable` asserts < 15 s instead of < 1.5 s | It runs in debug mode on CI where Argon2 is 10–15× slower; the real number is recorded in `docs/kdf-tuning.md` | Tighten once CI runs release tests |
| 22 | Release profile reduced to `panic = "abort"` + `strip = true` (no LTO, default codegen-units/opt-level) | With `lto = true` or `"thin"` on this machine (macOS 27.0.1, rustc 1.90) the full release build fails with `E0463 can't find crate for <proc-macro>` for whichever proc-macro dylib was just linked (`phf_macros`, `derive_more_impl`, `zerofrom_derive`, `tauri_macros`); bisected by building single crates with each profile variant. Binary is 10 MB, dmg 5 MB — fine. | Re-add `lto` once a newer toolchain/macOS no longer shows the bug |
| 23 | The macOS `.app` is not kept next to the `.dmg` after bundling; smoke test mounts the dmg | Tauri's dmg step consumed the `.app` on this run | Owner drags the app from the dmg as usual |
