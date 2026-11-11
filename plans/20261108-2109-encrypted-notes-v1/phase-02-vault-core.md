# Phase 02 — Vault core crate

## Context links
- Parent plan: [plan.md](plan.md)
- Depends on: phase-01
- Docs/research: [research-01](research/research-01-stack-and-libs.md) (rusqlite/SQLCipher, Argon2 section)

## Overview
- Date: 2026-11-08
- Priority: high
- Implementation status: done
- Review status: reviewed; post-review fixes unreviewed
- Description: Pure-Rust `znotes-core` vault lifecycle: create/open/lock, Argon2id KDF, plaintext
  meta sidecar, schema + migrations, error type, and the automated "no plaintext on disk" test.
  No Tauri dependency. Everything in this phase is unit-testable with `tempfile`.

## Key insights
- Bundled SQLCipher build already has FTS5 and `SQLITE_TEMP_STORE=2` (research-01), so the
  search index and temp tables stay encrypted/in-memory without extra config.
- Raw 64-hex key via `PRAGMA key = "x'…'"` skips SQLCipher's PBKDF2; Argon2id is our KDF, so
  unlock cost is Argon2 time + file open. Wrong key → `SQLITE_NOTADB` on first read.
- SQLite's default rollback journal (`vault.znotes-journal`) is also encrypted by SQLCipher. Keep
  `journal_mode=DELETE` (single process, single file for backups) and `synchronous=FULL`.
- Argon2 salt + params must be stored somewhere plaintext → `vault.meta.json` sidecar. It
  contains no secrets (salt, m/t/p, format version, created_at).
- Hold no raw key after `PRAGMA key`; wrap the derived key in `zeroize::Zeroizing` so it is wiped
  when dropped. Password change derives a new key with a fresh salt, `PRAGMA rekey`, then rewrites meta atomically.

## Requirements
- R1: `Vault::create(dir, password)` creates `vault.znotes` + `vault.meta.json`; fails with `AlreadyExists` if either exists.
- R2: `Vault::open(dir, password)` returns `Vault` on correct password; `Err(Error::WrongPassword)` on wrong; `Err(Error::NotAVault)` if meta missing/corrupt.
- R3: `vault.change_password(old, new)` → re-open with `new` works, with `old` fails.
- R4: Schema v1 applied on create; `schema_version` in `meta` table; open on a newer-version vault → `Error::UnsupportedVersion`.
- R5: Leak test: after creating a vault, writing a note containing a random marker, and dropping
  the vault, no file under the temp dir contains the marker bytes.
- R6: `Error` serializes (serde) to `{ code, message }` with stable snake_case codes.
- R7: `PRAGMA cipher_memory_security = ON` and `PRAGMA temp_store = MEMORY` set on every open.

## Architecture
```
crates/znotes-core/src/
├── lib.rs        pub use {Vault, Error, Result, VaultInfo}; pub mod repo (phase 03)
├── error.rs      enum Error { WrongPassword, NotAVault, AlreadyExists, UnsupportedVersion{found,supported},
│                              Locked, NotFound{entity,id}, InvalidInput(String), Io(#[from] io::Error), Db(#[from] rusqlite::Error) }
│                 impl Serialize → {"code": "wrong_password", "message": "..."}   (thiserror + serde)
├── kdf.rs        struct KdfParams { m_cost_kib: u32, t_cost: u32, p_cost: u32 }  Default = 65536/3/1
│                 fn derive(password: &[u8], salt: &[u8;16], p: &KdfParams) -> Result<Zeroizing<[u8;32]>>
│                 fn to_hex_key(&[u8;32]) -> Zeroizing<String>   // "x'…'"
├── meta.rs       struct VaultMeta { format_version: u32 (=1), kdf: KdfParams, salt_b64: String, created_at: i64 }
│                 load(dir) / save_atomic(dir, &meta)   // write .tmp then rename
├── schema.rs     const SCHEMA_VERSION: u32 = 1; fn migrate(conn) -> Result<()>
└── vault.rs      pub struct Vault { conn: Connection, dir: PathBuf }
                  create / open / change_password / path() / conn() (pub(crate)) / into_inner-free: Drop closes
```
Vault file layout: `<dir>/vault.znotes` (SQLCipher DB), `<dir>/vault.meta.json`.

Schema v1 (all inside the encrypted DB):
```sql
CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);            -- schema_version
CREATE TABLE folders(
  id INTEGER PRIMARY KEY, parent_id INTEGER REFERENCES folders(id) ON DELETE CASCADE,
  name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 200),
  created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL,
  UNIQUE(parent_id, name));
CREATE TABLE notes(
  id INTEGER PRIMARY KEY, folder_id INTEGER REFERENCES folders(id) ON DELETE CASCADE,
  title TEXT NOT NULL DEFAULT 'Untitled', body TEXT NOT NULL DEFAULT '',
  created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL);
CREATE INDEX notes_folder_updated ON notes(folder_id, updated_at DESC);
CREATE VIRTUAL TABLE notes_fts USING fts5(title, body, content='notes', content_rowid='id',
  tokenize='unicode61 remove_diacritics 2');
-- external-content sync triggers: notes_ai / notes_ad / notes_au (standard FTS5 pattern)
PRAGMA foreign_keys = ON;   -- set per connection on open
```
Open sequence: `PRAGMA key` → `PRAGMA cipher_memory_security=ON` → `PRAGMA temp_store=MEMORY` →
`PRAGMA foreign_keys=ON` → `PRAGMA synchronous=FULL` → `SELECT count(*) FROM sqlite_master` (key check) → `migrate()`.

## Related code files
| File | Role | Change |
|---|---|---|
| `crates/znotes-core/Cargo.toml` | add argon2, zeroize, serde, serde_json, thiserror, rand, base64, hex, tempfile(dev) | modify |
| `crates/znotes-core/src/lib.rs` | exports | modify |
| `crates/znotes-core/src/error.rs` | error enum + Serialize | NEW |
| `crates/znotes-core/src/kdf.rs` | Argon2id + hex key | NEW |
| `crates/znotes-core/src/meta.rs` | sidecar read/write | NEW |
| `crates/znotes-core/src/schema.rs` | DDL + migrate | NEW |
| `crates/znotes-core/src/vault.rs` | lifecycle | NEW |
| `crates/znotes-core/tests/vault_lifecycle.rs` | create/open/wrong pw/change pw/version | NEW |
| `crates/znotes-core/tests/leak.rs` | marker scan over temp dir | NEW |

New dependency justification: `argon2`+`zeroize` (KDF & key hygiene, no stdlib equivalent);
`thiserror`/`serde`/`serde_json` (error contract crosses IPC); `rand` (salt); `base64`, `hex`
(encodings); `tempfile` (tests only).

## Implementation steps
1. `error.rs`: define `Error`, `Result<T>`, `impl Serialize` producing `{code, message}`; unit test the code strings.
2. `kdf.rs`: `derive()` with `argon2::Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::new(m, t, p, Some(32)))`
   and `hash_password_into`; `to_hex_key()`. Test: same input → same key; different salt → different key; output 64 hex chars.
3. `meta.rs`: serde struct, `load` (map parse errors → `NotAVault`), `save_atomic` via `<file>.tmp` + `fs::rename`. Test round-trip + corrupt JSON.
4. `schema.rs`: `migrate(conn)` reads `meta.schema_version` (0 if table missing), applies v1 DDL + FTS triggers in one transaction, writes version. Returns `UnsupportedVersion` if stored > `SCHEMA_VERSION`.
5. `vault.rs`: `create()` → fresh salt, derive, `Connection::open(path)`, open sequence, `migrate`, write meta last (so a crash mid-create leaves no meta → `NotAVault`, user retries).
   `open()` → load meta, derive, open sequence; map `SqliteFailure(NotADb)` → `WrongPassword`.
   `change_password()` → derive new (fresh salt), `PRAGMA rekey`, `save_atomic(meta)`.
6. `tests/vault_lifecycle.rs`: R1–R4 scenarios incl. empty password rejected (`InvalidInput`), unicode password, meta tampered (`salt_b64` garbage → `NotAVault`).
7. `tests/leak.rs`: create vault in `tempfile::tempdir()`, insert a note row directly via `conn()` with body `format!("ZNOTES_LEAK_{}", random_hex)`, drop vault, walk dir recursively, read every file as bytes, assert marker absent; also assert `vault.znotes` size > 0 and `-journal` absent.
8. `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` clean.

## Todo list
- [x] error.rs with serde contract + test
- [x] kdf.rs Argon2id + tests
- [x] meta.rs atomic sidecar + tests
- [x] schema.rs v1 DDL + FTS triggers + version gate
- [x] vault.rs create/open/change_password/Drop
- [x] tests/vault_lifecycle.rs
- [x] tests/leak.rs
- [x] clippy + fmt clean

## Success criteria
- [x] `cargo test -p znotes-core` → `test result: ok.` with ≥ 12 passing, 0 failed, 0 ignored (33 passed across lib + 2 integration binaries)
- [x] `cargo clippy -p znotes-core -- -D warnings` exits 0
- [x] `strings <tempdir>/vault.znotes | grep ZNOTES_LEAK` returns nothing (covered by `tests/leak.rs`)
- [x] `hexdump -C vault.znotes | head -1` does NOT start with `SQLite format 3` (automated in `vault_file_does_not_start_with_sqlite_magic`)
- [x] Open with correct password on this machine completes in < 1.5 s (`cargo test -- --nocapture` prints elapsed) — 1.39 s in debug build; release will be faster, tuned in Phase 06

## Risk assessment
| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Wrong-password error not reliably `NotADb` across SQLCipher versions | low | medium | match on both `NotADb` and generic `SqliteFailure` after key check; test both paths |
| Crash between `rekey` and meta rewrite → vault unopenable | low | high | write new meta to `.tmp` first; on open, if `.tmp` exists try it as fallback; test the sequence |
| Argon2 64 MiB too slow on old Ubuntu laptop | medium | low | params in meta; Phase 06 tunes; `create()` takes `KdfParams` |
| External-content FTS triggers drift from table | low | medium | `INSERT INTO notes_fts(notes_fts) VALUES('rebuild')` helper + integrity test in Phase 03 |

## Security considerations
- Password bytes and derived key wrapped in `Zeroizing`; never logged; `Debug` impl for `Vault` omits them.
- Meta sidecar is plaintext by design; it reveals only that a vault exists and its KDF cost.
- No `PRAGMA cipher_plaintext_header_size`; the file must not start with the SQLite magic.
- Reject empty passwords; do not enforce composition rules (user's own vault).
- All DDL/DML uses parameter binding; no string-built SQL except the fixed `PRAGMA key` which is hex-only (validated 64 chars).

## Next steps
Phase 03 adds `repo::{folders, notes, search}` on top of `Vault::conn()` and exposes everything via Tauri commands.
