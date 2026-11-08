# Phase 05 — Preview, backup, change password, export, settings

## Context links
- Parent plan: [plan.md](plan.md)
- Depends on: phase-04
- Docs/research: [research-01](research/research-01-stack-and-libs.md) (VACUUM INTO note)

## Overview
- Date: 2026-11-08
- Priority: medium
- Implementation status: pending
- Review status: not-reviewed
- Description: Rounds out v1 features: sanitized markdown preview toggle, encrypted backup (file
  copy), change password, explicit plaintext export of one note, and a small plaintext settings
  file (last vault dir, autolock minutes).

## Key insights
- Backup = byte copy of `vault.znotes` + `vault.meta.json` while holding the vault mutex (no
  concurrent writes, rollback journal absent when idle). Avoids the unverified `VACUUM INTO`
  key semantics. Restore = user copies files back; document it.
- Preview renders our own markdown in our own webview, but a pasted snippet could contain
  `<script>`/`<img onerror>`; sanitize with DOMPurify anyway. Tauri's IPC is capability-gated,
  so impact is bounded, but defence in depth is cheap.
- Settings must not hold anything sensitive: last vault *path* is acceptable (file name is not secret), autolock minutes, preview on/off.
- Export writes plaintext by user choice; the dialog copy must say so.

## Requirements
- R1: Preview toggle (⌘/Ctrl+E) shows rendered markdown beside/instead of editor; GFM tables, code blocks, task lists render; `<script>` in a note does not execute.
- R2: "Backup vault…" saves `znotes-backup-YYYYMMDD-HHmm.znotes` + `.meta.json` to a chosen dir; result toast shows path; vault stays open.
- R3: "Change password…" requires current + new (twice); wrong current → error; after success relaunch opens only with new.
- R4: "Export note as .md…" shows warning text "This writes an unencrypted file" and requires confirm; writes `title.md` with body.
- R5: Settings: `autolock_minutes` (1–120, default 10) applied immediately to watchdog; `last_vault_dir` pre-fills Unlock.
- R6: Settings file lives in Tauri app config dir as `settings.json`; corrupt file → defaults, not crash.

## Architecture
```
Rust
crates/znotes-core/src/vault.rs       + pub fn backup_to(&self, dest_dir) -> Result<BackupPaths>
                                      + change_password already from Phase 02
src-tauri/src/commands/vault.rs       + vault_backup(dest_dir), vault_change_password(old,new), note_export_md(id, dest_path)
src-tauri/src/settings.rs             Settings{autolock_minutes:u32, last_vault_dir:Option<String>, preview:bool}; load/save (atomic) in app_config_dir
src-tauri/src/commands/settings.rs    settings_get(), settings_set(Settings)  → updates AppState.autolock
Frontend
src/lib/components/Preview.svelte     marked + DOMPurify → {@html}
src/lib/components/Menu.svelte        ⋯ menu: Backup, Change password, Export note, Settings, Lock
src/lib/components/ChangePassword.svelte, SettingsDialog.svelte, ExportConfirm.svelte
```
Added IPC: `vault_backup(dest_dir) -> {db_path, meta_path}`, `vault_change_password(old,new) -> ()`,
`note_export_md(id, dest_path) -> ()`, `settings_get() -> Settings`, `settings_set(s) -> Settings`.

## Related code files
| File | Role | Change |
|---|---|---|
| `crates/znotes-core/src/vault.rs` | `backup_to` | modify |
| `crates/znotes-core/tests/vault_lifecycle.rs` | backup opens with same pw | modify |
| `src-tauri/src/settings.rs` | settings persistence | NEW |
| `src-tauri/src/commands/{vault,settings}.rs` | new commands | modify / NEW |
| `src-tauri/src/state.rs` | `autolock` setter | modify |
| `src-tauri/src/watchdog.rs` | read autolock each tick | modify |
| `src-tauri/capabilities/default.json` | `dialog:allow-save` | modify |
| `src/lib/api.ts` | new wrappers | modify |
| `src/lib/components/{Preview,Menu,ChangePassword,SettingsDialog,ExportConfirm}.svelte` | UI | NEW |
| `src/lib/components/Workspace.svelte`, `TitleBar.svelte`, `Unlock.svelte` | wire menu/toggle/prefill | modify |
| `src/lib/markdown.ts` + `markdown.test.ts` | render+sanitize | NEW |
| `package.json` | add `marked`, `dompurify` | modify |

New dependency justification: `marked` (CommonMark+GFM renderer, no DOM needed) and `dompurify`
(sanitizer). Writing either by hand is a security bug waiting to happen.

## Implementation steps
1. `vault.rs::backup_to`: `fs::copy` db + meta into `dest_dir` with timestamped names; refuse if `-journal` exists (return `InvalidInput("write in progress")`). Test: backup dir opens with same password and contains the note.
2. `settings.rs`: serde struct with defaults; `load(app)` returns defaults on any error; `save_atomic`. Test with `tempfile` by passing the dir explicitly.
3. Commands + `AppState.autolock` store (AtomicU64 seconds); watchdog reads it each tick.
4. `note_export_md`: `repo::notes::get` → write `body` to `dest_path` (plugin-dialog `save()` on the frontend picks the path). Sanitize filename suggestion from title (`[^\w\- ]` → `_`).
5. `markdown.ts`: `render(md) = DOMPurify.sanitize(marked.parse(md, {gfm:true}), {USE_PROFILES:{html:true}})`; strip `target`/`javascript:`; test that `<script>` and `onerror` are removed and a table renders.
6. `Preview.svelte` with `{@html render(body)}`; toggle state in `workspace`; ⌘/Ctrl+E.
7. `Menu.svelte` + dialogs (inline modal components, no `window.confirm`). Change-password form validates new===repeat and non-empty.
8. `Unlock.svelte` prefill from `settings_get().last_vault_dir`; `settings_set` after successful open/create.
9. `pnpm check`, `pnpm test`, `cargo test --workspace`.

## Todo list
- [x] `backup_to` + test
- [x] settings.rs + test
- [x] 5 new commands + capability
- [x] markdown.ts + sanitizer tests
- [x] Preview toggle
- [x] Menu + ChangePassword + Settings + ExportConfirm dialogs
- [x] Unlock prefill
- [x] autolock applied live

## Success criteria
- [x] `cargo test --workspace` → all `ok`, includes backup-opens test (`backup_is_an_encrypted_copy_that_opens_with_the_same_password`)
- [x] `pnpm test` → includes `markdown.test.ts`: `<script>` removed, `onerror` removed, table rendered (37 passed)
- [x] `pnpm check` exits 0
- [ ] Manual: note containing `<img src=x onerror="document.title='pwned'">` → preview shows nothing harmful, window title unchanged
- [x] Manual: backup → `strings backup.znotes | grep -c <known word>` → `0`; opening backup dir in app works with same password (automated in core test; UI path covered by IPC test `backup_change_password_export_and_settings_through_ipc`)
- [x] Manual: change password → relaunch → old password "Wrong password", new opens (automated in IPC test; UI walkthrough left to owner)
- [ ] Manual: set autolock 1 min → idle 75 s → locked

## Risk assessment
| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Backup copied mid-transaction | low | high | copy under vault mutex; refuse when journal file present; test |
| User exports plaintext by accident | medium | medium | confirm dialog with explicit wording; export is not a shortcut |
| `marked` version changes output HTML | low | low | tests assert semantics (tag present/absent), not exact HTML |
| Settings file written to a synced dir | low | low | contains no secrets by design |

## Security considerations
- Preview HTML passes through DOMPurify; `{@html}` is used only there.
- Export is the single intentional plaintext path; it is explicit, confirmed, and logged nowhere.
- Change password: verify the old password first by deriving its key and opening a second, read-only connection to the vault file (fails with `WrongPassword` if wrong); only then derive the new key with a fresh salt, `PRAGMA rekey`, and rewrite meta atomically.
- Settings file is plaintext; never add password hints or note titles to it.

## Next steps
Phase 06 audits the whole app for plaintext leaks, tunes Argon2, and ships installers.
