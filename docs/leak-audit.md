# Leak audit — does any plaintext reach the disk?

This is the acceptance test of the project's one promise. A hit is a release blocker.

## Tooling
- `scripts/leak-check.sh <marker> [vault_dir]` greps the vault dir plus every location the
  app, its webview or the OS could write to (app data/config/cache dirs, WebKit profile dir,
  macOS saved state and crash reports, the user's temp dir). Exit 0 = clean.
- `cargo run -p znotes-core --example leak_fixture -- <dir> <marker>` creates a vault
  (password `pw`) whose only note contains the marker several times, without the UI.

## Protocol (run on every OS before a release)
1. Pick a marker nobody would type by accident: `M=ZNOTES_AUDIT_$(openssl rand -hex 4)`.
2. Launch the release build (`pnpm tauri build`, then open the bundle) — not `tauri dev`.
3. Create a vault in a fresh folder `V`. Create a folder and a note. Type the marker in the
   title and three times in the body, including a line `API_KEY=sk-$M`.
4. Toggle preview (⌘E), search for the marker (⌘K), switch notes, wait > 1 s for the save.
5. Lock (⌘L), unlock again, confirm the note is intact. Do **not** export.
6. Quit the app completely.
7. `scripts/leak-check.sh "$M" "$V"` → must print `RESULT: marker not found anywhere`.
8. Record the run in the table below. On a hit, the path tells you who wrote it.

## Mitigations already in place
- Vault, journal and FTS index are SQLCipher pages; `tests/leak.rs` proves no marker survives in the vault dir.
- `PRAGMA temp_store = MEMORY` and `SQLITE_TEMP_STORE=2`: no SQLite temp files.
- Webview window runs `incognito: true` (non-persistent WKWebsiteDataStore / WebKitGTK ephemeral data manager), so page cache, localStorage and IndexedDB are not written to disk.
- No note content is placed in web storage by the frontend; settings file holds no note data.
- Export to `.md` is the single intentional plaintext path and sits behind a warning dialog.

## Results
| Date | OS | Commit | Build | Result | Notes |
|---|---|---|---|---|---|
| 2026-11-08 | macOS 27.0.1 arm64 | (this commit) | fixture via `leak_fixture` + `scripts/leak-check.sh` | clean | Backend path only: the autonomous run had no display, so the UI steps 2–6 were not performed. Owner must run the full protocol. |
| | Ubuntu 24.04 | | | **not run** | Owner must run after Phase 01 gate. |
