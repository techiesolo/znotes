# Encrypted offline markdown notes app: stack & storage

- Date: 2026-11-08
- Status: decided

## Problem

- Personal, offline, Obsidian-like markdown notes app for macOS + Ubuntu.
- Hard requirement: nothing on disk is plaintext. Obsidian stores `.md` in the clear.
- Threat model:
  - Lost/stolen machine, disk backups, cloud folder backups.
  - Corporate endpoint/DLP agents that scan files on disk by keyword (e.g. `API_KEY`).
  - Explicitly **out of scope** (cannot be defended by any desktop app): screenshot capture
    by DLP agents while the app is unlocked, keyloggers, root-level process memory dumps.
- Scope v1: markdown editing, folder tree, full-text search. No wiki-links, no attachments.
- Unlock: password prompt on every launch; key lives in RAM only; auto-lock after idle.
- No sync. One vault per machine. No Obsidian import, proprietary format is fine.
- Scale: hundreds to a few thousand text notes.
- Team: one developer, learning project. Stack is free to choose.

## Options considered

### Baseline — don't build: Obsidian + gocryptfs / Cryptomator
- Pros: zero code, audited crypto, works today.
- Cons: while the vault is mounted, files are plaintext on the mount point and agents can
  grep them. Does not cover the "while in use" case. User also wants to learn by building.
- Decision: rejected, but it is the bar any built option must beat.

### Option A — Tauri v2 + Rust + SQLCipher (single encrypted DB)  ← chosen
- Whole vault = one SQLCipher file (AES-256-CBC per page + HMAC). FTS5 index lives inside
  the DB, so search index is encrypted for free. Note titles, folder structure, timestamps
  all hidden.
- Password → Argon2id → 32-byte raw key → `PRAGMA key = "x'...'"`. Salt + KDF params stored
  in a small plaintext header/sidecar (not secret).
- Frontend: Svelte 5 + CodeMirror 6 (markdown mode) in the webview. Rust side exposes
  vault/crypto/search via Tauri commands.
- Pros: near-zero hand-written crypto (no nonce management, no key wrapping, no custom
  file format). Atomic writes and crash safety from SQLite. ~5–10 MB bundle, low RAM.
- Cons: Rust learning curve. `rusqlite` with `bundled-sqlcipher-vendored-openssl`
  compiles OpenSSL → slow first build. Single file → need a backup story.
- Risk: WebKitGTK rendering bugs on Ubuntu (NVIDIA / Wayland). Tauri docs provide env-var
  workarounds (`WEBKIT_DISABLE_DMABUF_RENDERER`, `__NV_DISABLE_EXPLICIT_SYNC=1`,
  `GDK_BACKEND=x11`). Must be validated on the actual Ubuntu machine before committing.

### Option B — Tauri v2 + Rust + per-note encrypted files (XChaCha20-Poly1305 / age)
- Each note = one file named by random UUID, content + metadata encrypted. FTS rebuilt in
  memory at unlock.
- Pros: sync-friendly (cloud folder / git), one corrupt file ≠ lost vault.
- Cons: much larger self-written crypto surface (file format, nonces, key wrapping,
  encrypted manifest, atomic rename). FTS rebuild on every unlock.
- Rejected: its main benefit (sync) is explicitly not needed. YAGNI.

### Option C — Electron + SQLCipher (same storage design as A)
- Pros: bundled Chromium → no WebKitGTK risk on Ubuntu; one language; mature ecosystem.
- Cons: 100–300 MB bundle, 3–5× RAM of Tauri. Irrelevant for personal use, but no upside
  over A if Tauri renders fine.
- Kept as the **fallback** if the Ubuntu spike fails. Storage layer design is identical.

### Rejected — native UI (egui / iced / Flutter / Compose)
- A good markdown editor is the hardest part of the app; CodeMirror 6 is years of work
  that would have to be re-implemented. Not worth it.

## Decision

Option A: Tauri v2 + Rust backend + SQLCipher single-file vault + Svelte 5 + CodeMirror 6.

Why it won: it minimises the amount of cryptography we write ourselves, gives encrypted
full-text search and hidden metadata without extra design, and keeps the app small.
Option C is the pre-agreed fallback, switched to only if the Ubuntu rendering spike fails.

## Implementation considerations

- **Phase 0 (spike, first week)**: Tauri v2 hello-world + CodeMirror 6 on the real Ubuntu
  machine. Check NVIDIA/Wayland rendering. Go/no-go gate for Tauri vs Electron.
- **Vault format**: `vault.znotes` (SQLCipher DB) + `vault.znotes.meta` (JSON: Argon2id
  salt/params, format version). Tables: `folders`, `notes`, `notes_fts` (FTS5, external
  content). All in one DB.
- **Key handling**: Argon2id (memory-hard, tuned to ~0.5–1 s on target machines). Derived
  key held in Rust only, wrapped in `zeroize`. Never passed to the webview. Auto-lock after
  N minutes idle closes the connection and zeroizes the key.
- **Password change**: `PRAGMA rekey`. No re-encryption code to write.
- **No plaintext leakage paths** (this is where real failures happen, not the cipher):
  - Never store note content in webview localStorage / IndexedDB / sessionStorage.
  - No temp files; drafts live in RAM until written to the DB.
  - Export to `.md` is an explicit user action with a warning.
  - Give the vault a custom extension so OS indexers do not try to parse it; verify
    Spotlight does not index it.
  - Disable webview spellcheck/form autofill persistence if the platform writes them to
    disk.
- **Backup**: "Export encrypted backup" = copy of the DB file (safe anywhere, it is
  ciphertext). Add SQLite `PRAGMA integrity_check` on open.
- **Build**: `rusqlite` features `bundled-sqlcipher-vendored-openssl`, `fts5`. CI matrix:
  macOS (arm64) + Ubuntu 24.04 (`.deb` depends on `libwebkit2gtk-4.1`).
- Sequencing after spike: vault create/open/lock → CRUD notes & folders → editor → FTS
  search → auto-lock → backup/export → packaging.

## Risks & mitigations

- **WebKitGTK renders badly on Ubuntu** → env-var workarounds; if still broken, switch to
  Electron (Option C), storage layer unchanged.
- **Forgot password = vault unrecoverable** → by design. Document it loudly; offer an
  optional printed recovery key (second wrapped copy of the DB key) in a later version.
- **Single-file corruption** → SQLite is crash-safe (WAL/rollback journal); add integrity
  check on open and the encrypted-backup command early.
- **Plaintext leaks via webview or OS** → the checklist above; add an automated test that
  greps the whole app data dir and vault dir for a known marker string after a session.
- **Screenshot/keylogger/memory-dump agents** → accepted, out of scope. Auto-lock reduces
  the exposure window. Do not pretend otherwise in UI copy.
- **Argon2id params too heavy on an old laptop** → make params tunable and stored in the
  meta file.
- **Long Rust/OpenSSL compile times** → cache `target/` in CI; acceptable for one dev.

## Success metrics

- `grep -r` / `strings` over the vault directory and app data directory never finds any
  note text or title after a full edit session. (Automated test.)
- Unlock in < 1.5 s on both machines with Argon2id tuned; search over 5k notes < 100 ms.
- App runs on the developer's macOS and Ubuntu machines without rendering glitches.
- Vault file copied to another machine opens with the same password.

## Open questions

- Exact Argon2id parameters (memory/iterations) after measuring on both machines.
- Whether to offer a recovery key in v1 or defer.
- CodeMirror 6 vs a WYSIWYG-ish markdown editor (Milkdown/Tiptap) — decide in zk:plan
  after trying CM6 in the spike.
- Whether WKWebView (macOS) / WebKitGTK persist spellcheck or form data to disk; verify
  during the leakage test.
