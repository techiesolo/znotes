# Phase 04 — Frontend: unlock, folders, notes, editor, search

## Context links
- Parent plan: [plan.md](plan.md)
- Depends on: phase-03
- Docs/research: [research-01](research/research-01-stack-and-libs.md) (CodeMirror, Tauri mocks)

## Overview
- Date: 2026-11-08
- Priority: high
- Implementation status: pending
- Review status: not-reviewed
- Description: Usable app: create/unlock screen, three-pane layout (folder tree · note list ·
  editor), CodeMirror 6 markdown editing with debounced save, search box, lock button, and
  reaction to `vault-locked`. Svelte 5 runes, no router, no UI framework.

## Key insights
- Keep CM6 direct (Phase 01 already did); `EditorView.updateListener` + 500 ms debounce → `note_update`.
  Flush pending save on note switch, window blur, lock, and `beforeunload`.
- Note content must never go into `localStorage`/`sessionStorage`/IndexedDB; only UI prefs
  (selected folder id, pane widths) may be persisted, and even those stay in memory for v1 (YAGNI).
- Vault directory selection uses `@tauri-apps/plugin-dialog` `open({directory: true})`; needs
  `dialog:allow-open` capability.
- All IPC goes through one typed module `src/lib/api.ts` mirroring Phase 03's table; tests use
  `mockIPC` from `@tauri-apps/api/mocks` with jsdom.

## Requirements
- R1: On start, call `vault_status`; show Unlock screen (choose dir, password, Open / Create tabs). Wrong password shows "Wrong password", clears field, keeps focus.
- R2: After unlock: folder tree (nested, collapsible, context actions new/rename/delete), note list for selected folder (title + relative updated time), editor for selected note.
- R3: Typing saves after 500 ms idle; status shows "Saved"/"Saving…"; switching notes with unsaved text saves first.
- R4: Title is an input above the editor; empty → "Untitled" on save.
- R5: Search box (⌘/Ctrl+K focuses) lists hits with snippet; Enter/click opens the note.
- R6: Lock button and `vault-locked` event both return to Unlock screen and clear all in-memory note state.
- R7: Keyboard: ⌘/Ctrl+N new note, ⌘/Ctrl+L lock.
- R8: System light/dark via `prefers-color-scheme`; CM6 theme follows.

## Architecture
```
src/
├── main.ts, App.svelte                     # App: status → <Unlock/> | <Workspace/>
├── app.css                                  # tokens, light/dark
└── lib/
    ├── api.ts                               # typed invoke wrappers + types from Phase 03 table + listenLocked()
    ├── state/vault.svelte.ts                # $state: status, dir; unlock(), create(), lock()
    ├── state/workspace.svelte.ts            # $state: folders, notes, selectedFolderId, selectedNoteId, currentNote, saveState
    ├── utils/debounce.ts, utils/time.ts
    └── components/
        ├── Unlock.svelte                    # dir picker (dialog), password, create/open
        ├── Workspace.svelte                 # 3-pane grid + shortcuts
        ├── FolderTree.svelte, FolderNode.svelte
        ├── NoteList.svelte
        ├── Editor.svelte                    # CM6 (from Phase 01, now bound to currentNote)
        ├── TitleBar.svelte                  # title input + save status + lock button
        └── Search.svelte
```
Data flow: component → `workspace.*()` action → `api.*()` → invoke → state updated from the
returned entity (no optimistic edits except editor text).

## Related code files
| File | Role | Change |
|---|---|---|
| `src/lib/api.ts` | IPC layer | NEW |
| `src/lib/state/vault.svelte.ts`, `workspace.svelte.ts` | state | NEW |
| `src/lib/components/*.svelte` (8 files) | UI | NEW / `Editor.svelte` modify |
| `src/App.svelte`, `src/app.css` | shell | modify |
| `src/lib/utils/debounce.ts`, `time.ts` | helpers | NEW |
| `src/lib/api.test.ts`, `src/lib/state/workspace.test.ts`, `src/lib/components/Unlock.test.ts`, `Search.test.ts` | tests | NEW |
| `vite.config.ts` | vitest `environment: 'jsdom'`, `@testing-library/jest-dom` setup | modify |
| `package.json` | add `@tauri-apps/plugin-dialog`, dev: vitest, jsdom, @testing-library/svelte, jest-dom | modify |
| `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs` | register `tauri-plugin-dialog` | modify |
| `src-tauri/capabilities/default.json` | `dialog:allow-open` | modify |

## Implementation steps
1. `api.ts`: one exported function per command, `invoke<T>(name, args)`; `AppError` type guard on `{code}`; `onVaultLocked(cb)` via `listen`. Test with `mockIPC` that args are passed by name and errors propagate.
2. `vault.svelte.ts`: `status: 'loading'|'locked'|'open'`; `refresh()`, `open(dir,pw)`, `create(dir,pw)`, `lock()`; subscribe to `vault-locked` → `status='locked'` + call `workspace.reset()`.
3. `workspace.svelte.ts`: load folders/notes, select, `createNote`, `updateDraft(title, body)` → debounced `flush()`; `flush()` awaited on `selectNote`, `lock`, `blur`. Test: debounce coalesces 3 edits into 1 `note_update`; switching notes flushes first.
4. `Unlock.svelte`: Open/Create tabs; `open({directory:true})` from plugin-dialog; disable button while deriving; map `wrong_password`/`not_a_vault`/`already_exists` codes to messages. Test: wrong password renders message and clears input.
5. `FolderTree.svelte`/`FolderNode.svelte`: build tree from flat list; inline rename; delete asks for confirm via a small inline "Delete? Yes/No" (no `window.confirm` — blocks webview automation).
6. `NoteList.svelte`, `TitleBar.svelte`, `Editor.svelte`: bind to `workspace`; `Editor` replaces doc when `selectedNoteId` changes (`view.dispatch({changes:{from:0,to:len,insert}})`), not re-mounting.
7. `Search.svelte`: 200 ms debounce → `api.search(q, 50)`; render snippets as text (snippet markers `[`/`]` → `<mark>`, escape everything else). Test: hits rendered, click selects note.
8. `Workspace.svelte`: CSS grid `240px 280px 1fr`; global keydown for ⌘/Ctrl+K/N/L; `window.addEventListener('blur', flush)`.
9. `app.css`: tokens + `@media (prefers-color-scheme: dark)`; CM6 `EditorView.theme` reading the same CSS vars.
10. Register dialog plugin in Rust, add capability, `pnpm check`, `pnpm test`.

## Todo list
- [ ] api.ts + tests
- [ ] vault state + locked-event handling
- [ ] workspace state + debounce/flush tests
- [ ] Unlock screen + tests
- [ ] FolderTree with create/rename/delete
- [ ] NoteList + TitleBar + Editor binding
- [ ] Search + tests
- [ ] Shortcuts, blur flush, light/dark
- [ ] dialog plugin + capability

## Success criteria
- [ ] `pnpm check` exits 0 (svelte-check, 0 errors)
- [ ] `pnpm test` → vitest `Tests  N passed (N)`, N ≥ 12, 0 failed
- [ ] `cargo build --workspace` exits 0 (dialog plugin wired)
- [ ] Manual: create vault → new folder → new note → type "tiếng việt" → wait 1 s → relaunch app → unlock → note present; search "tieng" finds it
- [ ] Manual: ⌘/Ctrl+L returns to Unlock; reopening shows the same content
- [ ] Manual: devtools Application tab shows empty localStorage/IndexedDB after a session

## Risk assessment
| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Debounced save lost on quit | medium | medium | flush on `blur`, `beforeunload`, and Tauri `CloseRequested` (Rust emits, frontend flushes, then confirms close) |
| CM6 doc replace resets undo history across notes | certain | low | acceptable v1; per-note `EditorState` cache is a follow-up |
| jsdom lacks `ResizeObserver`/layout CM6 wants | medium | low | don't test `Editor.svelte` in jsdom; test state logic instead |
| Svelte 5 + testing-library API drift | low | low | pin versions from research-01 |

## Security considerations
- Snippets and titles rendered as text nodes; the only HTML is our own `<mark>`.
- No note content in web storage; `vault-locked` clears `workspace` state (`currentNote = null`, arrays emptied).
- Password input `type=password`, `autocomplete="off"`; value cleared after submit.
- CSP from Phase 01 kept; no remote fonts/scripts.

## Next steps
Phase 05 adds preview, backup, change password, export and settings on top of this shell.
