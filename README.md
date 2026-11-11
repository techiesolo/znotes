# znotes

Offline markdown notes for macOS and Ubuntu, **encrypted at rest**. Think of a tiny Obsidian
whose vault is one SQLCipher file instead of a folder of plaintext `.md` files.

## What it protects against, honestly

| Threat | Covered? | How |
|---|---|---|
| Lost or stolen laptop, disk image, cloud backup of the vault folder | yes | Notes, titles, folder names and the search index live inside one AES-256 SQLCipher file; the key is derived from your password with Argon2id (128 MiB, t=4). |
| Agents on the machine that grep files for keywords (`API_KEY`, …) | yes | Nothing readable is written: no plaintext temp files, no web storage, webview runs in incognito mode. `scripts/leak-check.sh` verifies this. |
| Someone shoulder-surfing or a screenshot/screen-recording agent while the app is unlocked | **no** | Nothing on a desktop can stop this. Auto-lock (default 10 min) shortens the window. |
| Keyloggers, or a root-level agent dumping the process memory | **no** | Out of scope by design. |
| Forgetting your password | **no** | There is no recovery. Keep a backup of the password somewhere safe. |

Export to `.md` is the one intentional plaintext path and sits behind a warning dialog.

## Install

Unsigned builds are attached to each GitHub release.

- **macOS (Apple Silicon)**: open the `.dmg`, drag `znotes.app` to Applications, then run once
  `xattr -d com.apple.quarantine /Applications/znotes.app` because the app is not notarized.
- **Ubuntu 24.04**: `sudo apt install ./znotes_*.deb` (pulls `libwebkit2gtk-4.1-0`), or run the
  `.AppImage` directly. On NVIDIA/Wayland the app sets `WEBKIT_DISABLE_DMABUF_RENDERER=1` itself;
  if the window is still blank try `__NV_DISABLE_EXPLICIT_SYNC=1 znotes` or `GDK_BACKEND=x11 znotes`.

Verify downloads against the SHA-256 sums printed in the release.

## Using it

- **Create vault**: pick an empty folder and a password. The folder gets `vault.znotes`
  (ciphertext) and `vault.meta.json` (Argon2 salt and parameters, not secret).
- Folders, notes, search (`⌘/Ctrl+K`, diacritics-insensitive: `tieng` finds `tiếng`), new note
  `⌘/Ctrl+N`, preview `⌘/Ctrl+E`, lock `⌘/Ctrl+L`.
- **Backup** (menu ⋯): copies the encrypted vault and its meta file to a folder of your choice.
  Restore by copying both files back into an empty folder and opening it with the same password.
- **Change password** re-encrypts the whole vault in place.
- Settings: auto-lock minutes. Stored in plaintext in the OS config dir, never contains note data.

## Development

```sh
pnpm install
pnpm tauri dev          # app with hot reload
pnpm check && pnpm test # svelte-check, vitest
cargo test --workspace  # vault, repos, IPC tests
pnpm tauri build        # .dmg / .deb / .AppImage
```

Linux prerequisites: `libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev
libayatana-appindicator3-dev librsvg2-dev`.

Layout: `crates/znotes-core` (vault, KDF, schema, repositories, search; no Tauri dependency),
`src-tauri` (thin command layer, auto-lock watchdog, settings), `src` (Svelte 5 + CodeMirror 6).
Design history lives in `docs/decisions/`, the plan in `plans/`, choices made during the
autonomous build in `docs/decisions-log.md`, and the leak audit protocol in `docs/leak-audit.md`.
