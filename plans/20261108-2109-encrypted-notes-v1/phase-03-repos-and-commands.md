# Phase 03 — Repositories, Tauri commands, state, auto-lock

## Context links
- Parent plan: [plan.md](plan.md)
- Depends on: phase-02
- Docs/research: [research-01](research/research-01-stack-and-libs.md) (Tauri testing, capabilities)

## Overview
- Date: 2026-11-08
- Priority: high
- Implementation status: done
- Review status: reviewed; post-review fixes unreviewed
- Description: CRUD for folders and notes, FTS5 search, and the IPC surface the frontend will
  call. `AppState` holds at most one open vault behind a mutex; an idle watchdog locks it and
  emits `vault-locked`.

## Key insights
- Keep `src-tauri` commands one-liners that lock the state and delegate to `znotes-core::repo`,
  so almost all logic is testable with plain `cargo test` and the Tauri mock runtime is optional.
- rusqlite `Connection` is `Send` but not `Sync`; `Mutex<Option<Vault>>` in `tauri::State` is the
  right shape. Commands are sync (`fn`, not `async`) so Tauri runs them on its thread pool and the
  mutex guard never crosses an await.
- FTS5 external-content tables need the standard three triggers; `remove_diacritics 2` makes
  "tieng viet" match "tiếng việt". Prefix search via `token*`. User input is passed as a bound
  parameter but FTS5 query syntax still parses it → wrap each token in double quotes.
- Idle lock belongs in Rust (authoritative) with the frontend as a mirror: a UI timer alone can be
  defeated by a frozen webview.

## Requirements
- R1: Folders: list (flat, with parent_id), create, rename, delete (cascades to subfolders and notes). Duplicate name in same parent → `InvalidInput`.
- R2: Notes: list by folder (summaries only, no body), get, create, update(title, body), move, delete. Unknown id → `NotFound`.
- R3: `search(q, limit)` returns hits with FTS5 `snippet()` and `bm25` ordering; empty/whitespace query → empty vec; query "tieng" matches a note containing "tiếng".
- R4: Every command on a locked vault returns `{code:"locked"}`; frontend can rely on this.
- R5: Idle watchdog: if no command for `autolock_minutes` (default 10), vault is dropped and
  `vault-locked` event emitted; `vault_status` reflects it.
- R6: Only `vault_create`/`vault_open`/`vault_status` work while locked.

## Architecture
```
crates/znotes-core/src/repo/
├── mod.rs       pub mod folders; pub mod notes; pub mod search; pub(crate) fn now() -> i64
├── folders.rs   Folder{id,parent_id,name,created_at,updated_at}; list/create/rename/delete(conn,…)
├── notes.rs     NoteSummary{id,folder_id,title,updated_at}; Note{…,body,created_at}; list/get/create/update/move_to/delete
└── search.rs    SearchHit{id,folder_id,title,snippet}; fn search(conn,q,limit) ; fn build_fts_query(q)->String ; fn rebuild_index(conn)

src-tauri/src/
├── lib.rs       run(): manage(AppState), register commands, spawn watchdog
├── state.rs     pub struct AppState { vault: Mutex<Option<Vault>>, last_activity: Mutex<Instant>, autolock: AtomicU64 }
│                fn with_vault<T>(&self, f: impl FnOnce(&Vault)->Result<T>) -> Result<T>   // touches last_activity, maps None→Locked
├── commands/mod.rs, vault.rs, folders.rs, notes.rs, search.rs
└── watchdog.rs  tauri::async_runtime::spawn loop every 15 s: if idle > autolock → take() vault, emit "vault-locked"
```
IPC contract (all return `Result<T, znotes_core::Error>`; names are the `invoke` strings):
| Command | Args | Returns |
|---|---|---|
| `vault_create` | `dir: String, password: String` | `VaultInfo{dir}` |
| `vault_open` | `dir, password` | `VaultInfo` |
| `vault_lock` | — | `()` |
| `vault_status` | — | `VaultStatus{locked: bool, dir: Option<String>}` |
| `folder_list` | — | `Vec<Folder>` |
| `folder_create` | `parent_id: Option<i64>, name: String` | `Folder` |
| `folder_rename` | `id, name` | `Folder` |
| `folder_delete` | `id` | `()` |
| `note_list` | `folder_id: Option<i64>` | `Vec<NoteSummary>` |
| `note_get` | `id` | `Note` |
| `note_create` | `folder_id: Option<i64>, title: String` | `Note` |
| `note_update` | `id, title, body` | `Note` |
| `note_move` | `id, folder_id: Option<i64>` | `Note` |
| `note_delete` | `id` | `()` |
| `search` | `query: String, limit: u32` | `Vec<SearchHit>` |
Event: `vault-locked` (payload `{reason: "idle" | "manual"}`).

## Related code files
| File | Role | Change |
|---|---|---|
| `crates/znotes-core/src/repo/{mod,folders,notes,search}.rs` | data access | NEW |
| `crates/znotes-core/src/lib.rs` | `pub mod repo` | modify |
| `crates/znotes-core/tests/repo_folders.rs`, `repo_notes.rs`, `repo_search.rs` | tests | NEW |
| `src-tauri/src/state.rs` | AppState | NEW |
| `src-tauri/src/commands/*.rs` | IPC layer | NEW |
| `src-tauri/src/watchdog.rs` | idle lock | NEW |
| `src-tauri/src/lib.rs` | wiring | modify |
| `src-tauri/Cargo.toml` | add `znotes-core`, `serde`, `tauri` feature `test` (dev) | modify |
| `src-tauri/capabilities/default.json` | `core:default`, `core:event:allow-listen` | modify |

## Implementation steps
1. `repo/folders.rs`: implement with prepared statements; map `UNIQUE` violation → `InvalidInput("folder name already exists")`. Tests: nested create, rename collision, cascade delete removes notes.
2. `repo/notes.rs`: `update` sets `updated_at`; `list` orders by `updated_at DESC`; `move_to(None)` = root. Tests incl. `NotFound`, 1 MB body round-trip, title trimmed, empty title → `Untitled`.
3. `repo/search.rs`: `build_fts_query` → split on whitespace, strip `"`, wrap each token `"tok"*`, join with space (implicit AND). `search` SQL:
   `SELECT n.id, n.folder_id, n.title, snippet(notes_fts, 1, '[', ']', '…', 12) FROM notes_fts JOIN notes n ON n.id = notes_fts.rowid WHERE notes_fts MATCH ?1 ORDER BY bm25(notes_fts) LIMIT ?2`.
   Tests: diacritics, prefix, multi-token AND, quote injection (`a" OR "b`) returns no error, empty query → empty.
4. `state.rs`: `with_vault` + `set_vault` + `take_vault`; unit test `Locked` path without Tauri.
5. `commands/*.rs`: `#[tauri::command] fn note_get(state: State<AppState>, id: i64) -> Result<Note, Error> { state.with_vault(|v| repo::notes::get(v.conn(), id)) }` pattern for all rows above.
6. `watchdog.rs`: spawn in `setup`; reads `autolock` (seconds) from `AppState`; emits via `app.emit("vault-locked", …)`.
7. `lib.rs`: `.manage(AppState::default())`, `.invoke_handler(tauri::generate_handler![…])`, `.setup(|app| { watchdog::spawn(app.handle().clone()); Ok(()) })`.
8. Optional (if `tauri::test::mock_builder` exists in 2.12 as documented): one test calling `vault_status` through `get_ipc_response` to lock the serialized error shape `{"code":"locked"}`. Otherwise assert the JSON shape in `znotes-core` tests only and note it in the plan.

## Todo list
- [x] folders repo + tests
- [x] notes repo + tests
- [x] search repo + query builder + tests (diacritics, injection)
- [x] AppState with `with_vault`
- [x] all 15 commands registered
- [x] watchdog emits `vault-locked`
- [x] capabilities updated
- [x] (optional) mock-runtime IPC test — `src-tauri/tests/ipc.rs`, 2 tests through the real command layer

## Success criteria
- [x] `cargo test --workspace` → `test result: ok.` for every crate, ≥ 30 passing total, 0 failed (63 passed)
- [x] `cargo clippy --workspace -- -D warnings` exits 0
- [x] `pnpm tauri dev` then in devtools console: `await __TAURI__.core.invoke('vault_status')` → `{locked: true, dir: null}` (verified by `tests/ipc.rs::locked_app_reports_status_and_rejects_data_commands` via mock runtime instead of devtools)
- [ ] With a vault open and autolock set to 1 min for the test, after 75 s `vault_status` → `locked: true` and the `vault-locked` event fired (manual, devtools)

## Risk assessment
| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| FTS5 query syntax error on odd user input | medium | low | quote every token; test `*`, `-`, `(`, `:` inputs |
| Mutex held across a slow command blocks UI | low | low | commands are short; no I/O besides SQLite |
| `tauri::test` API differs from docs | medium | low | optional step 8; core tests carry coverage |
| Watchdog races a command mid-write | low | medium | watchdog takes the same mutex; SQLite transactions complete before drop |

## Security considerations
- All SQL parameterised; FTS MATCH input tokenised and quoted.
- Capabilities grant only `core:default` + event listen; no `fs`/`shell` plugins.
- `Error::Db` messages are passed to the UI; ensure they never include note content (rusqlite errors don't).
- Lock drops the `Vault` → SQLCipher frees and (with memory_security) wipes key material.

## Next steps
Phase 04 consumes this IPC table verbatim; `src/lib/api.ts` is generated by hand from it.
