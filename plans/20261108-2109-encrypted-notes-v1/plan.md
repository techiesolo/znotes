# znotes v1 — encrypted offline markdown notes (macOS + Ubuntu)

- Created: 2026-11-08 21:09
- Mode: hard
- Source: [docs/decisions/20261108-encrypted-markdown-desktop-notes.md](../../docs/decisions/20261108-encrypted-markdown-desktop-notes.md)
- Research: [research/research-01-stack-and-libs.md](research/research-01-stack-and-libs.md)

## Goal
A Tauri v2 desktop app (Svelte 5 + CodeMirror 6 frontend, Rust backend) that stores all
notes, folders and the search index inside one SQLCipher-encrypted vault file. Password on
every launch, Argon2id-derived raw key, auto-lock. Nothing readable by `grep` ever touches
disk. Ships as `.dmg` (macOS arm64) and `.deb` + `.AppImage` (Ubuntu 24.04).

## Scope
In:
- Vault create / open / lock / change password; one vault open at a time.
- Folders (nested) and markdown notes with title + body; full-text search (FTS5, diacritics-insensitive).
- CodeMirror 6 markdown editor with debounced save; sanitized read-only preview toggle.
- Auto-lock after idle; encrypted backup = copy of vault file; export one note to `.md` (explicit, warned).
- Leak audit script + automated leak test; CI build matrix; release packaging.

Out (deliberately):
- Sync of any kind, multi-vault, wiki-links/backlinks, attachments/images, Obsidian import.
- OS keychain / biometric unlock, recovery key, plugins, themes beyond system light/dark.
- Windows build. Mobile.

## Phases
| # | Phase | Status | File |
|---|---|---|---|
| 01 | Scaffold & platform spike (Tauri + Svelte + CM6 + SQLCipher compiles, runs on Ubuntu) | done | [phase-01-scaffold-and-spike.md](phase-01-scaffold-and-spike.md) |
| 02 | Vault core crate (KDF, meta sidecar, open/create/lock, schema, leak test) | done | [phase-02-vault-core.md](phase-02-vault-core.md) |
| 03 | Repositories + Tauri commands + state + auto-lock | done | [phase-03-repos-and-commands.md](phase-03-repos-and-commands.md) |
| 04 | Frontend: unlock, folder tree, note list, editor, search | done | [phase-04-frontend.md](phase-04-frontend.md) |
| 05 | Preview, backup, change password, export, settings | done | [phase-05-preview-backup-settings.md](phase-05-preview-backup-settings.md) |
| 06 | Leak audit, Argon2 tuning, packaging & release CI | done | [phase-06-hardening-and-release.md](phase-06-hardening-and-release.md) |

Status: pending | done | blocked

## Architecture in one picture
```
 Svelte 5 UI ── invoke() ──▶ src-tauri (thin commands, AppState{Mutex<Option<Vault>>})
   CM6 editor                      │ uses
                                   ▼
                          crates/znotes-core  (no tauri dep)
                          kdf · meta · vault · schema · repo{folders,notes,search}
                                   │ rusqlite + bundled SQLCipher
                                   ▼
                   <dir>/vault.znotes (ciphertext)  +  <dir>/vault.meta.json (salt, KDF params)
```

## Open questions
1. SQLCipher `cipher_memory_security` and raw-key semantics recalled from memory; confirm against
   zetetic docs in Phase 02 before relying on them.
2. Exact `tauri::test` mock-runtime API names for command tests (Phase 03). Fallback: keep
   commands as 1-line wrappers and test only `znotes-core`.
3. Ubuntu machine GPU/session type unknown; Phase 01 gate decides Tauri vs Electron fallback.
4. Argon2id final parameters depend on measurements on both machines (Phase 06).
5. Preview renderer: `marked` + `DOMPurify` chosen; could be replaced by CM6 inline decorations
   later if the two-pane UX feels wrong.
