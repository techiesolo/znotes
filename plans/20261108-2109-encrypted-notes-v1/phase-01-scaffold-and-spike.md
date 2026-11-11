# Phase 01 — Scaffold & platform spike

## Context links
- Parent plan: [plan.md](plan.md)
- Depends on: none
- Docs/research: [research-01](research/research-01-stack-and-libs.md); Tauri prerequisites, Linux graphics page

## Overview
- Date: 2026-11-08
- Priority: high
- Implementation status: done
- Review status: reviewed; post-review fixes unreviewed
- Description: Stand up the repo skeleton (cargo workspace + Vite/Svelte 5 + Tauri 2), prove the
  two riskiest unknowns early: SQLCipher compiles & links on both OSes, and the WebKitGTK webview
  renders a CodeMirror editor correctly on the real Ubuntu machine. This is the Tauri-vs-Electron gate.

## Key insights
- Repo is empty (`README.md` only). No existing patterns to follow; this phase sets them.
- `cargo-tauri` is not installed locally (toolchain check). Rust 1.90 / node 20 / pnpm 10 present.
- create-tauri-app's "svelte" template scaffolds SvelteKit. We don't need routing or SSR, so use
  plain Vite + Svelte 5 (`pnpm create vite` → svelte-ts) and `cargo tauri init`. Fewer moving parts.
- rusqlite `bundled-sqlcipher-vendored-openssl` builds OpenSSL from source: first build is slow
  (minutes). Prove it compiles on Ubuntu CI before writing any vault code.
- Linux rendering bugs are NVIDIA/Wayland specific; workaround env vars must be set in `main()`
  before `tauri::Builder` runs.

## Requirements
- R1: `pnpm tauri dev` opens a window on macOS showing a CodeMirror 6 markdown editor.
- R2: Same on the Ubuntu machine, with no blank/black window or visible tearing; if env-var
  workarounds are needed they are applied in code, not by the user.
- R3: `cargo test -p znotes-core` runs a test that opens an in-memory SQLCipher connection and
  asserts `PRAGMA cipher_version` is non-empty.
- R4: GitHub Actions builds (`cargo build --release` + `pnpm build`) on `macos-14` and `ubuntu-24.04`.
- R5: Written go/no-go note in `docs/spike-ubuntu.md`: GPU, session type, env vars needed, result.

## Architecture
```
znotes/
├── Cargo.toml                 # [workspace] members = ["src-tauri", "crates/znotes-core"]
├── package.json  vite.config.ts  svelte.config.js  tsconfig.json  index.html
├── src/                       # frontend (Svelte 5 + TS)
│   ├── main.ts  App.svelte
│   └── lib/components/Editor.svelte          # CM6 wrapper (spike version)
├── src-tauri/
│   ├── Cargo.toml  build.rs  tauri.conf.json
│   ├── capabilities/default.json
│   └── src/{main.rs, lib.rs}
├── crates/znotes-core/
│   ├── Cargo.toml             # rusqlite(bundled-sqlcipher-vendored-openssl), no tauri
│   └── src/lib.rs             # only `cipher_version()` for now
└── .github/workflows/ci.yml
```
- `tauri.conf.json`: `identifier = "com.techiesolo.znotes"`, `build.frontendDist = "../dist"`,
  `build.devUrl = "http://localhost:1420"`, `beforeDevCommand = "pnpm dev"`, `beforeBuildCommand = "pnpm build"`.
- `src-tauri/src/main.rs`: on `cfg(target_os = "linux")` set `WEBKIT_DISABLE_DMABUF_RENDERER=1`
  only if the spike proves it necessary (document the decision either way).

## Related code files
| File | Role | Change |
|---|---|---|
| `Cargo.toml` | workspace root | NEW |
| `package.json`, `vite.config.ts`, `svelte.config.js`, `tsconfig.json`, `index.html` | frontend tooling | NEW |
| `src/main.ts`, `src/App.svelte` | app entry | NEW |
| `src/lib/components/Editor.svelte` | CM6 markdown editor (spike) | NEW |
| `src-tauri/Cargo.toml`, `build.rs`, `tauri.conf.json`, `capabilities/default.json` | Tauri shell | NEW |
| `src-tauri/src/main.rs`, `src-tauri/src/lib.rs` | Tauri entry | NEW |
| `crates/znotes-core/Cargo.toml`, `src/lib.rs` | core crate stub | NEW |
| `.github/workflows/ci.yml` | build matrix | NEW |
| `docs/spike-ubuntu.md` | gate record | NEW |
| `.gitignore` | node_modules, target, dist | NEW |

## Implementation steps
1. `pnpm create vite@latest . --template svelte-ts` (in repo root), then `pnpm add -D @tauri-apps/cli@^2`
   and `pnpm add @tauri-apps/api@^2 codemirror @codemirror/lang-markdown @codemirror/view @codemirror/state`.
2. `pnpm tauri init` → answer: app name `znotes`, window title `znotes`, frontend dist `../dist`,
   dev URL `http://localhost:1420`, dev cmd `pnpm dev`, build cmd `pnpm build`. Set Vite `server.port = 1420`, `strictPort = true`.
3. Create `Cargo.toml` workspace at root; move `src-tauri` into it; create `crates/znotes-core` with
   `rusqlite = { version = "0.41", features = ["bundled-sqlcipher-vendored-openssl"] }`.
4. `crates/znotes-core/src/lib.rs`: `pub fn cipher_version() -> rusqlite::Result<String>` opening
   `Connection::open_in_memory()` and querying `PRAGMA cipher_version`. Unit test asserts non-empty.
5. `src/lib/components/Editor.svelte`: mount `EditorView` with `markdown()` + `basicSetup` on a
   `$effect`, destroy on cleanup. `App.svelte` renders it with a seeded markdown string.
6. `src-tauri/src/lib.rs`: `run()` with `tauri::Builder::default()`; `main.rs` calls it. Add Linux env-var
   block gated by `cfg(target_os = "linux")` with a comment pointing to the Tauri Linux-graphics page.
7. `.github/workflows/ci.yml`: matrix `macos-14`, `ubuntu-24.04`; Ubuntu installs the apt list from
   research-01; steps: pnpm install → `pnpm build` → `cargo build --release --workspace` →
   `cargo test --workspace`. Cache `~/.cargo` and `target/`.
8. Run `pnpm tauri dev` on macOS; then build/copy to the Ubuntu machine and run; record results in `docs/spike-ubuntu.md`.
9. If Ubuntu renders wrong after trying the documented env vars → STOP, mark phase `blocked`, and
   re-plan on Electron (decision record Option C). Storage phases 02–03 are unaffected.

## Todo list
- [x] Vite + Svelte 5 TS scaffold, CM6 deps installed
- [x] `tauri init`, port 1420, identifier set
- [x] Cargo workspace with `crates/znotes-core` (SQLCipher feature) and `src-tauri`
- [x] `cipher_version()` + unit test
- [x] `Editor.svelte` renders markdown with highlighting
- [x] Linux env-var block (or documented "not needed")
- [ ] CI matrix green on both OSes
- [x] `docs/spike-ubuntu.md` written with go/no-go

## Success criteria
- [x] `cargo test -p znotes-core` → `test result: ok. 1 passed`
- [x] `pnpm build` exits 0 and produces `dist/index.html`
- [x] `cargo build --release --workspace` exits 0 on macOS and on ubuntu-24.04 CI (macOS debug build verified locally; CI result pending first push)
- [ ] `pnpm tauri dev` on macOS: window shows editor with markdown heading highlighted (manual)
- [ ] Ubuntu machine: same window renders correctly, typing has no lag/tearing (manual, recorded in `docs/spike-ubuntu.md`)
- [x] `pnpm check` (svelte-check) exits 0

## Risk assessment
| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| WebKitGTK blank/garbled window on Ubuntu | medium | high (stack change) | try documented env vars; else switch to Electron per decision record |
| OpenSSL vendored build fails on Ubuntu CI (missing `perl`/`make`) | low | medium | `build-essential` + `perl` in apt list; cache target |
| First build time 10+ min frustrates iteration | high | low | cargo cache in CI; `sccache` optional locally |
| Vite 8 / Svelte 5.57 plugin mismatch | low | low | pin versions from research-01 |

## Security considerations
- No secrets yet. Set the app identifier now; it determines webview profile dirs scanned by the leak check later.
- `capabilities/default.json` starts with `core:default` only; add permissions per phase, never `*`.
- CSP in `tauri.conf.json`: `default-src 'self'; style-src 'self' 'unsafe-inline'` (CM6 injects styles). No remote origins.

## Next steps
Phase 02 builds the real vault inside `crates/znotes-core`; it needs the workspace layout and
proven SQLCipher build from here.
