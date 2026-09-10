//! The wickra-zk guest program — the code that runs *inside* the zkVM and
//! whose honest execution a receipt attests to.
//!
//! Contract (see `docs/ZK.md` and `docs/DETERMINISM.md`):
//!
//! 1. Read the private inputs: the strategy spec, the candle series, and the
//!    prover-supplied `dataset_commitment`.
//! 2. Bind the data: recompute the canonical hash of the candles and assert it
//!    equals the committed value, so the receipt cannot be reused for other
//!    data than the one it commits to.
//! 3. Run the exact same deterministic backtest the native `wickra-backtest`
//!    engine runs.
//! 4. Hash the report with the exact same canonicalization `wickra-proof`
//!    uses natively, so `journal.report_hash` is byte-identical to the value
//!    the rest of the ecosystem computes.
//! 5. Commit only the public journal — the private data and strategy never
//!    leave the guest.
//!
//! Building this needs a risc0 toolchain (see CONTRIBUTING.md); `cargo build -p
//! wickra-zk-methods` drives it through `risc0-build`. It does not need a
//! `no_std` build of the engine: risc0's guest std shim carries what
//! `wickra-backtest` and `wickra-proof` use, and both compile for
//! `riscv32im-risc0-zkvm-elf` as published.

#![no_main]

use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};
use wickra_backtest::{Candle, StrategySpec};

risc0_zkvm::guest::entry!(main);

/// The journal committed by the guest and read back by the host as
/// `wickra_zk_host::GuestJournal` (same field names, order and types).
///
/// Metrics are rounded to a fixed number of decimals before committing so the
/// journal is stable across platforms; the rounding matches the host's
/// `round_to` in `wickra-zk-host::model`. The image id is NOT committed here —
/// a guest cannot know its own id; the host fills it after decoding.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GuestJournal {
    /// Canonical `wickra-proof` hash of the backtest report — the same hash the
    /// native path and every language binding produce.
    report_hash: String,
    /// Canonical hash of the candle series the proof was computed over.
    dataset_commitment: String,
    sharpe: f64,
    pnl: f64,
    n_trades: u64,
}

/// Round to 8 decimals. Bit-for-bit the host's `round_to(x, 1e-8)`, which is
/// the same three operations in the same order: scale up, round, scale back.
/// The host recomputes these figures and compares them to the ones committed
/// here, so "close enough" is not enough -- the two expressions have to produce
/// the identical `f64`.
fn round8(x: f64) -> f64 {
    (x * 1e8).round() / 1e8
}

fn main() {
    // 1. Private inputs, in the order the host writes them in `prove.rs`.
    let strategy: StrategySpec = env::read();
    let candles: Vec<Candle> = env::read();
    let dataset_commitment: String = env::read();

    // 2. Bind the data to the commitment.
    let recomputed =
        wickra_proof_core::hash_candles(&candles).expect("candles must serialise to canonical JSON");
    assert_eq!(
        recomputed, dataset_commitment,
        "dataset_commitment does not match the candles fed to the guest"
    );

    // 3. Deterministic backtest — spec first, then candles (engine order).
    let report = wickra_backtest::run(&strategy, &candles)
        .expect("backtest must succeed for a valid spec/data pair");

    // 4. Canonical report hash, identical to the native wickra-proof hash.
    let report_hash =
        wickra_proof_core::hash_report(&report).expect("report must serialise to canonical JSON");

    // 5. Commit the journal only (host adds the image id after decoding).
    // The headline figures live on `report.metrics`, not on the report itself.
    let journal = GuestJournal {
        report_hash,
        dataset_commitment,
        sharpe: round8(report.metrics.sharpe),
        pnl: round8(report.metrics.pnl),
        n_trades: u64::try_from(report.metrics.num_trades)
            .expect("a trade count cannot exceed u64"),
    };
    env::commit(&journal);
}
