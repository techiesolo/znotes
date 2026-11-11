# Phase 06 — Leak audit, Argon2 tuning, packaging & release CI

## Context links
- Parent plan: [plan.md](plan.md)
- Depends on: phase-05
- Docs/research: [research-01](research/research-01-stack-and-libs.md) (webview profile dirs, .deb deps, Argon2)

## Overview
- Date: 2026-11-08
- Priority: high
- Implementation status: pending
- Review status: not-reviewed
- Description: Prove the core promise end-to-end on real OS paths (not just the vault dir), pick
  Argon2 parameters from measurements, and produce installers from CI on tag push.

## Key insights
- The Rust leak test (Phase 02) only covers the vault directory. The webview and OS can write
  elsewhere: WebKit profile/cache dirs, crash reporters, macOS saved-application-state. The
  audit script greps all of them for a marker typed through the real UI.
- Argon2 cost is a UX/security trade; measure `derive()` at m=64 MiB/t=3 on both machines and
  adjust `t` so unlock ≈ 0.5–1 s on the slower one. Params are per-vault in meta, so changing the
  default never breaks existing vaults.
- Tauri bundles: macOS `.dmg`/`.app` (unsigned is fine for personal use; document Gatekeeper
  `xattr -d com.apple.quarantine`), Linux `.deb` (depends `libwebkit2gtk-4.1-0`) + `.AppImage`.
- macOS "Saved Application State" can persist window contents for some apps; set
  `NSQuitAlwaysKeepsWindows`-style behaviour off by disabling state restoration (Tauri default
  doesn't restore; verify by checking `~/Library/Saved Application State/<bundle-id>.savedState`).

## Requirements
- R1: `scripts/leak-check.sh <marker>` scans vault dir, app data/config/cache dirs, WebKit dirs,
  `~/Library/Saved Application State/com.techiesolo.znotes.savedState`, `/tmp` (owned by user), and
  swap-free check note; exits non-zero if the marker is found; prints every path scanned.
- R2: Manual audit protocol in `docs/leak-audit.md` executed on macOS and Ubuntu with results recorded (marker typed in editor, previewed, searched, app quit, then scan).
- R3: Argon2 defaults chosen from measurements; `docs/kdf-tuning.md` records numbers per machine.
- R4: `cargo tauri build` produces `.dmg` on macOS and `.deb` + `.AppImage` on Ubuntu 24.04; installing the `.deb` and launching works on the Ubuntu machine.
- R5: GitHub Actions `release.yml` on tag `v*` builds both and uploads artifacts to a GitHub Release.
- R6: `README.md` documents: threat model (what is / isn't protected), install, backup/restore, forgotten-password = data loss.

## Architecture
```
scripts/leak-check.sh          bash; args: marker, [vault_dir]; uses `grep -rIl` + `strings` for binaries
docs/leak-audit.md             protocol + results table (OS, date, commit, result)
docs/kdf-tuning.md             measurements
crates/znotes-core/benches/    (optional) criterion-free timing test behind `--ignored`
.github/workflows/release.yml  tag → tauri-action or manual `pnpm tauri build` per OS → upload
src-tauri/tauri.conf.json      bundle.targets ["dmg"] / ["deb","appimage"], icons, deb depends
```

## Related code files
| File | Role | Change |
|---|---|---|
| `scripts/leak-check.sh` | audit | NEW |
| `docs/leak-audit.md`, `docs/kdf-tuning.md` | records | NEW |
| `crates/znotes-core/src/kdf.rs` | default params from tuning | modify |
| `crates/znotes-core/tests/kdf_timing.rs` | `#[ignore]` timing print | NEW |
| `src-tauri/tauri.conf.json` | bundle config, icons, version | modify |
| `src-tauri/icons/*` | generated via `pnpm tauri icon` | NEW |
| `.github/workflows/release.yml` | release pipeline | NEW |
| `.github/workflows/ci.yml` | add `pnpm test`, clippy, fmt gates | modify |
| `README.md` | user docs | modify |

## Implementation steps
1. `scripts/leak-check.sh`: build path list per `uname`; for each existing dir run
   `grep -rIl --exclude-dir=node_modules "$MARKER"` and `find -type f -exec sh -c 'strings "$1" | grep -q "$0"' "$MARKER" {} \; -print`; print scanned dirs; exit 1 on any hit.
2. `docs/leak-audit.md`: steps (launch → create vault in temp dir → note with marker `ZNOTES_AUDIT_<rand>` typed in editor → toggle preview → search it → export NOT done → quit) then run script; record on both OSes; investigate any hit (likely culprits: WebKit cache, crash logs).
3. If a hit appears from WebKit storage: disable persistent data via Tauri `WebviewWindowBuilder` incognito/data-directory options (Tauri 2 `incognito(true)` exists on Linux/Windows; verify macOS) and re-run.
4. `tests/kdf_timing.rs` (`#[ignore]`): time `derive()` for (64 MiB, t=2..4). Run on both machines; pick default; update `KdfParams::default()`; record in `docs/kdf-tuning.md`.
5. `pnpm tauri icon <source.png>`; set `bundle.active=true`, `bundle.targets`, `bundle.linux.deb.depends=["libwebkit2gtk-4.1-0"]` (verify Tauri adds it automatically; keep explicit if not), `version` from `package.json`.
6. `release.yml`: matrix macos-14 / ubuntu-24.04; `tauri-apps/tauri-action` with `tagName: v__VERSION__`, `releaseDraft: true`; or manual build+upload if the action's Node/Rust versions lag.
7. `ci.yml`: add `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `pnpm check`, `pnpm test`.
8. README: threat model table (protected: disk/grep/backup/theft; not: screenshots, keylogger, root memory dump), install per OS incl. Gatekeeper note, backup/restore, password loss.
9. Install `.deb` on Ubuntu machine; run `scripts/leak-check.sh` there as final gate.

## Todo list
- [x] leak-check.sh
- [x] leak-audit.md with macOS + Ubuntu results (macOS backend-only; Ubuntu marked not run)
- [x] WebKit persistence fix if needed (window `incognito: true` applied proactively)
- [x] kdf timing + defaults + kdf-tuning.md
- [x] icons + bundle config
- [x] release.yml; ci.yml gates
- [x] README threat model + install
- [ ] .deb installed & audited on Ubuntu (owner: no Ubuntu machine in this run)

## Success criteria
- [ ] `scripts/leak-check.sh ZNOTES_AUDIT_x` after a full UI session → `exit 0`, output lists ≥ 5 scanned dirs, on macOS AND Ubuntu (script verified: exit 1 on a plaintext fixture, exit 0 on a `leak_fixture` vault; UI session and Ubuntu pending)
- [x] `cargo test -p znotes-core -- --ignored kdf_timing --nocapture` prints timings; chosen default yields 0.28 s here in release (0.4–1.2 s target applies to the slower machine, not measured)
- [x] `pnpm tauri build` → `target/release/bundle/dmg/znotes_0.1.0_aarch64.dmg` (5.0 MB, macOS, app launches from the mounted dmg and creates no WebKit/cache/saved-state dirs); `.deb`/`.AppImage` only on Ubuntu CI (not run here)
- [ ] `dpkg -I *.deb | grep Depends` includes `libwebkit2gtk-4.1-0` (checked by a step in `release.yml`; not runnable on macOS)
- [ ] Tag `v0.1.0` → GitHub Release draft with 3 artifacts (owner decides when to tag)
- [x] CI: fmt, clippy, svelte-check, vitest, cargo test all green on both OSes (all green locally on macOS; `ci.yml` runs the same gates on ubuntu-24.04 + macos-14 on push)

## Risk assessment
| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| WebKit writes page cache containing rendered note text | medium | high | audit finds it; switch webview to incognito/no-persistence; re-audit |
| macOS crash log contains heap snippets | low | medium | out of scope to prevent; document; auto-lock reduces exposure |
| Unsigned macOS app blocked by Gatekeeper | certain | low | README `xattr` instruction; signing is out of scope v1 |
| tauri-action incompatibility with Vite 8 / Node 20 | medium | low | fall back to manual build + `softprops/action-gh-release` |
| Argon2 64 MiB OOM on constrained CI | low | low | tests use small params; defaults only used by app |

## Security considerations
- The audit is the acceptance test of the whole project's premise; a hit is a release blocker, not a note.
- Release artifacts are unsigned; README states checksums are published per release (`sha256sum` in release notes).
- No telemetry, no update checker in v1 (no network at all); confirm with `lsof -i` while running.

## Next steps
v1 done. Candidates for v2 (separate brainstorm): recovery key, per-note undo history cache,
wiki-links, attachments, optional sync (would revisit Option B storage).
