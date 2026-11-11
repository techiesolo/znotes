# Argon2id parameter tuning

Measured with `cargo test -p znotes-core --release --test kdf_timing -- --ignored --nocapture`.
Parameters are stored per vault in `vault.meta.json`, so changing the default never breaks an
existing vault; it only affects vaults created afterwards.

## macOS, Apple Silicon (2026-11-08, release build)
| memory | t=1 | t=2 | t=3 | t=4 |
|---|---|---|---|---|
| 32 MiB | 44 ms | 39 ms | 45 ms | 61 ms |
| 64 MiB | 35 ms | 64 ms | 98 ms | 131 ms |
| 128 MiB | 75 ms | 136 ms | 207 ms | **278 ms** |

## Ubuntu machine
Not measured yet (no access during the autonomous run). Run the command above on it and add a
row. If the default lands above ~1.5 s there, lower `t_cost` to 3 in `KdfParams::default()`.

## Chosen default
`m = 128 MiB, t = 4, p = 1` → ~0.28 s on Apple Silicon, expected ~0.6–1.0 s on a mid-range
x86 laptop. Above the OWASP minimum (19 MiB / t=2) by a wide margin while keeping peak memory
during unlock to 128 MiB, which an older laptop can afford.

Debug builds are roughly 10–15× slower (the `default_params_open_time_is_reasonable` test
prints its own number); that is why the test threshold is loose.
