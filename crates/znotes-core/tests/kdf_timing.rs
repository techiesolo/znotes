use std::time::Instant;

use znotes_core::KdfParams;
use znotes_core::kdf::{SALT_LEN, derive};

#[test]
#[ignore = "timing probe for docs/kdf-tuning.md; run with --release -- --ignored --nocapture"]
fn kdf_timing() {
    let salt = [3u8; SALT_LEN];
    for m_cost_kib in [32 * 1024, 64 * 1024, 128 * 1024] {
        for t_cost in [1, 2, 3, 4] {
            let params = KdfParams {
                m_cost_kib,
                t_cost,
                p_cost: 1,
            };
            let started = Instant::now();
            derive(b"benchmark-password", &salt, &params).unwrap();
            println!(
                "m={:>3} MiB t={} p=1 -> {:>7.0} ms",
                m_cost_kib / 1024,
                t_cost,
                started.elapsed().as_secs_f64() * 1000.0
            );
        }
    }
    let default = KdfParams::default();
    let started = Instant::now();
    derive(b"benchmark-password", &salt, &default).unwrap();
    println!(
        "DEFAULT m={} MiB t={} p={} -> {:.0} ms",
        default.m_cost_kib / 1024,
        default.t_cost,
        default.p_cost,
        started.elapsed().as_secs_f64() * 1000.0
    );
}
