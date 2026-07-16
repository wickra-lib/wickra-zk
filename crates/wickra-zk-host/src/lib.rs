//! Prover and verifier for **wickra-zk** — a trustless, data-hiding proof of
//! backtest performance.
//!
//! The host runs the deterministic `wickra-backtest` engine inside a risc0
//! zkVM guest and turns the resulting receipt into a [`ZkProof`]. A verifier
//! who never sees the price data or the strategy can still confirm that the
//! reported metrics are the honest output of the audited program:
//!
//! - [`prove`] executes the guest and returns a receipt plus [`PublicOutputs`].
//! - [`verify`] checks a receipt against the pinned [`guest_id`] and returns
//!   the public outputs.
//! - [`command_json`] is the string-in/string-out boundary every binding uses.
//!
//! The journal (`report_hash`, `dataset_commitment`, `sharpe`, `pnl`,
//! `n_trades`) is the only thing that leaves the zkVM; the candles and the
//! strategy stay private. `report_hash` is byte-identical to the hash
//! `wickra-proof` computes natively, tying a proof to the wider ecosystem.

pub use wickra_backtest::{BacktestReport, Candle, StrategySpec};

mod command;
mod error;
mod model;
mod prove;
mod verify;

pub use command::command_json;
pub use error::{Error, Result};
pub use model::{round_to, GuestJournal, ProveOptions, PublicOutputs, ZkProof, ZkSpec};
pub use prove::prove;
pub use verify::verify;

/// Hex-encoded image id of the embedded guest — the value a verifier pins.
#[must_use]
pub fn guest_id() -> String {
    risc0_zkvm::sha::Digest::from(wickra_zk_methods::WICKRA_ZK_GUEST_ID).to_string()
}

/// The `wickra-zk-host` crate version.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Compute the dataset commitment for a candle slice — the canonical hash the
/// guest recomputes and binds the proof to. Callers (e.g. the CLI) use this to
/// fill `ZkSpec::dataset_commitment` when a spec omits it.
///
/// Provisional: mirrors the guest's `wickra-proof` candle hash; the exact
/// function name settles once the no_std hashing path lands upstream.
#[must_use]
pub fn commit_dataset(candles: &[Candle]) -> String {
    wickra_proof::hash_candles(candles)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_outputs() -> PublicOutputs {
        PublicOutputs {
            report_hash: "a".repeat(64),
            dataset_commitment: "b".repeat(64),
            guest_id: "c".repeat(64),
            sharpe: round_to(1.234_567_891_2, 1e-8),
            pnl: round_to(1000.123_456_789, 1e-8),
            n_trades: 7,
        }
    }

    #[test]
    fn round_to_matches_wickra_proof() {
        assert!((round_to(1.234_567_891_2, 1e-8) - 1.234_567_89).abs() < 1e-12);
        assert!((round_to(-0.000_000_004, 1e-8)).abs() < 1e-12);
    }

    #[test]
    fn public_outputs_serde_roundtrip() {
        let out = sample_outputs();
        let json = serde_json::to_string(&out).unwrap();
        let back: PublicOutputs = serde_json::from_str(&json).unwrap();
        assert_eq!(out, back);
    }

    #[test]
    fn guest_journal_serde_roundtrip() {
        let journal = GuestJournal {
            report_hash: "a".repeat(64),
            dataset_commitment: "b".repeat(64),
            sharpe: 1.5,
            pnl: 42.0,
            n_trades: 3,
        };
        let json = serde_json::to_string(&journal).unwrap();
        let back: GuestJournal = serde_json::from_str(&json).unwrap();
        assert_eq!(journal, back);
    }

    #[test]
    fn from_journal_carries_guest_id() {
        let journal = GuestJournal {
            report_hash: "a".repeat(64),
            dataset_commitment: "b".repeat(64),
            sharpe: 1.0,
            pnl: 2.0,
            n_trades: 1,
        };
        let out = PublicOutputs::from_journal(journal, "deadbeef".to_string());
        assert_eq!(out.guest_id, "deadbeef");
        assert_eq!(out.report_hash, "a".repeat(64));
    }

    #[test]
    fn command_json_rejects_unknown_command() {
        let err = command_json(r#"{"cmd":"nope"}"#).unwrap_err();
        assert!(matches!(err, Error::Parse(_)));
    }

    #[test]
    fn command_json_version_reports_crate_version() {
        let resp = command_json(r#"{"cmd":"version"}"#).unwrap();
        assert!(resp.contains(version()));
    }
}
