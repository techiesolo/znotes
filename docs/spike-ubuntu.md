# Phase 01 spike — platform gate

## macOS (this machine)
- Date: 2026-11-08
- Machine: macOS 27.0.1, Apple Silicon (arm64)
- Result: see Phase 01 success criteria in the plan; `pnpm tauri dev` rendering is checked
  by the owner (the autonomous run cannot see a window).

## Ubuntu
- Status: **NOT RUN** during the autonomous `zk:cook` session (no Ubuntu machine reachable).
- What the code already does: `src-tauri/src/lib.rs` sets `WEBKIT_DISABLE_DMABUF_RENDERER=1`
  on Linux unless the variable is already set. This is the first workaround on the Tauri
  Linux-graphics page and is harmless on non-NVIDIA machines.
- Owner checklist on the Ubuntu machine:
  1. `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`
  2. `pnpm install && pnpm tauri dev`
  3. Record below: GPU vendor (`lspci | grep -i vga`), session (`echo $XDG_SESSION_TYPE`),
     whether the editor renders and typing is smooth.
  4. If the window is blank/garbled, retry with `__NV_DISABLE_EXPLICIT_SYNC=1` and then
     `GDK_BACKEND=x11`. If one fixes it, add it to `set_webkitgtk_env_before_threads_start`.
  5. If nothing fixes it: decision record Option C (Electron) applies; `crates/znotes-core`
     is reusable unchanged.

| Date | GPU | Session | Env vars needed | Renders OK | Notes |
|---|---|---|---|---|---|
| | | | | | |

## CI
- `.github/workflows/ci.yml` builds and tests on `ubuntu-24.04` and `macos-14`. A green Ubuntu
  job proves compilation and packaging dependencies, not rendering.
