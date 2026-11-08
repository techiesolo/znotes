# Research 01 — Stack & library facts (verified 2026-11-08)

Source of truth for versions/features referenced by the phase files. Anything marked
**VERIFY** could not be confirmed from primary docs during planning and must be checked in
the phase that uses it.

## Local toolchain (this macOS machine, arm64)
- rustc/cargo 1.90.0, node 20.19.5, pnpm 10.28.0. `cargo-tauri` NOT installed.
- Ubuntu machine: GPU / X11-vs-Wayland unknown → Phase 01 gate.

## Current versions (crates.io / npm, 2026-11-08)
| Dep | Version | Notes |
|---|---|---|
| tauri | 2.12.5 | tauri-build 2.7.4 |
| @tauri-apps/cli, @tauri-apps/api | 2.12.4 | |
| tauri-plugin-dialog / @tauri-apps/plugin-dialog | 2.8.4 | folder picker, save dialog |
| rusqlite | 0.41.1 | see features below |
| argon2 | 0.6.0 | Argon2id KDF |
| zeroize | 1.9.1 | wipe derived key |
| svelte | 5.57.3 | runes |
| vite | 8.3.6 | |
| vitest | 5.1.0 | + jsdom 31, @testing-library/svelte 5.4 |
| @codemirror/lang-markdown | 6.5.2 | @codemirror/view 6.43, state 6.7, `codemirror` 6.0.2 meta pkg |
| typescript | 7.0.2 | svelte-check 4.7.8 |
| @sveltejs/kit | 3.1.1 | NOT used — plain Vite+Svelte chosen (no routing/SSR needed) |

## rusqlite / SQLCipher
- Features (crates.io metadata, rusqlite 0.41.1):
  `bundled-sqlcipher-vendored-openssl` → `bundled-sqlcipher` → `bundled`. No `fts5` feature exists.
- libsqlite3-sys `build.rs` (master) for any bundled build adds
  `-DSQLITE_ENABLE_FTS3 -DSQLITE_ENABLE_FTS3_PARENTHESIS -DSQLITE_ENABLE_FTS5`; for
  `bundled-sqlcipher` also `-DSQLITE_HAS_CODEC -DSQLITE_TEMP_STORE=2
  -DSQLITE_EXTRA_INIT=sqlcipher_extra_init`. → FTS5 available, temp tables default to memory.
- Raw key: `PRAGMA key = "x'<64 hex chars>'"` must be the first statement; a 64-hex raw key
  bypasses SQLCipher's PBKDF2 (we do Argon2id ourselves). Wrong key surfaces as
  `SQLITE_NOTADB` ("file is not a database") on first read.
- `PRAGMA rekey = "x'...'"` re-encrypts in place.
- `PRAGMA cipher_memory_security = ON` wipes freed memory (off by default). **VERIFY** exact
  behaviour in SQLCipher 4.x bundled by rusqlite 0.41 (zetetic.net fetch failed during planning).
- `VACUUM INTO` with SQLCipher: key behaviour of the output file is **unclear** → backup is
  implemented as a byte copy of the vault file under the vault mutex instead.

## Tauri v2
- Linux prerequisites (official page): `libwebkit2gtk-4.1-dev build-essential curl wget file
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`.
- `.deb` runtime depends on `libwebkit2gtk-4.1-0` (official debian distribute page).
- Linux graphics issues (official page): NVIDIA/Wayland → env vars
  `WEBKIT_DISABLE_DMABUF_RENDERER=1`, `__NV_DISABLE_EXPLICIT_SYNC=1`, `GDK_BACKEND=x11`,
  `WEBKIT_DISABLE_COMPOSITING_MODE=1`. Can be set in `main()` before Tauri starts.
- Testing: Rust mock runtime behind tauri feature `test` (`tauri::test::mock_builder`,
  `mock_context`, `noop_assets`) **VERIFY** names in Phase 03. Frontend: `@tauri-apps/api/mocks`
  exposes `mockIPC`, `mockWindows`, `clearMocks`.
- Commands return `Result<T, E>` where `E: serde::Serialize`.
- Capabilities live in `src-tauri/capabilities/*.json`; need `core:default`,
  `dialog:allow-open`, `dialog:allow-save`.

## Argon2id parameters
- OWASP minimum: m=19 MiB, t=2, p=1. RFC 9106 second recommendation: m=64 MiB, t=3, p=4.
- Chosen default: m=64 MiB, t=3, p=1, output 32 bytes. Stored in the meta sidecar so it can be
  tuned per machine (Phase 06) without a format change.

## CodeMirror 6
- Use the packages directly (no Svelte wrapper): `EditorState.create`, `EditorView`,
  `markdown({ base: markdownLanguage })`, `EditorView.updateListener.of(...)` for change
  detection. Wrappers (`svelte-codemirror-editor`) add a dependency for ~30 lines of glue.

## Threat-model-relevant OS facts
- Proofpoint / Purview endpoint DLP can capture screenshots and scan file content on macOS and
  Linux. Encryption at rest defeats the file scan; nothing defeats screenshots → auto-lock
  only shortens the window.
- Webview profile dirs to include in the leak check:
  macOS `~/Library/WebKit/<bundle-id>`, `~/Library/Caches/<bundle-id>`,
  `~/Library/Application Support/<bundle-id>`; Linux `~/.local/share/<bundle-id>`,
  `~/.cache/<bundle-id>`, `~/.config/<bundle-id>`.

## Sources
- https://v2.tauri.app/start/prerequisites/
- https://v2.tauri.app/distribute/debian/
- https://v2.tauri.app/develop/debug/linux-graphics/
- https://v2.tauri.app/develop/tests/
- https://docs.rs/crate/rusqlite/0.41.1 , https://crates.io/api/v1/crates/rusqlite/0.41.1
- https://raw.githubusercontent.com/rusqlite/rusqlite/master/libsqlite3-sys/build.rs
- https://www.zetetic.net/sqlcipher/sqlcipher-api/ (fetch failed; recalled, VERIFY)
- https://docs.public.analyze.proofpoint.com/itm_endpoint_dlp_configuration/features_itm_mac_windows_linux.htm
