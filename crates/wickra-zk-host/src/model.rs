//! Data model for the prove/verify boundary (see docs/ZK.md §6).
//!
//! Two structs mirror the two halves of the journal:
//!
//! - [`GuestJournal`] is exactly what the guest commits inside the zkVM — the
//!   report hash, the echoed dataset commitment and the selected metrics. It
//!   carries no image id, because a guest cannot know its own image id.
//! - [`PublicOutputs`] is the host-facing view: a `GuestJournal` plus the
//!   `guest_id` the host pins from `WICKRA_ZK_GUEST_ID`, so a consumer sees in
//!   one place *what* was proven and *which* program proved it.

use serde::{Deserialize, Serialize};
use wickra_backtest::StrategySpec;

use crate::error::{Error, Result};

/// Round exactly like `wickra-proof` does before hashing/serialising a report:
/// `round_to(x, 1e-8) == (x * 1e8).round() / 1e8`.
///
/// Scale up, round, scale back down -- in that order. Dividing by `precision`
/// and multiplying back is the same arithmetic on paper and a different `f64`
/// in practice: for a pnl of 1483.61984049 it returns 1483.6198404900001, while
/// the guest's `round8` returns 1483.61984049. The guest commits its value into
/// the journal and the host recomputes its own, so the two have to agree
/// bit-for-bit, not to within an epsilon. The golden test's 1e-8 tolerance hid
/// the difference; the fixtures did not.
#[must_use]
pub fn round_to(value: f64, precision: f64) -> f64 {
    let scale = 1.0 / precision;
    (value * scale).round() / scale
}

/// The prove-time input: a private strategy plus a commitment to the data.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ZkSpec {
    /// Full `StrategySpec` as understood by `wickra-backtest`. Stays private —
    /// never committed to the journal.
    pub strategy: StrategySpec,
    /// Lower-case hex (64 chars) canonical hash of the candle series, so the
    /// proof is bound to specific data without revealing it. The guest
    /// recomputes this and panics on mismatch.
    pub dataset_commitment: String,
}

impl ZkSpec {
    /// Parse a `ZkSpec` from its JSON form.
    ///
    /// # Errors
    /// Returns [`Error::Parse`] if the JSON is not a valid `ZkSpec`.
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| Error::Parse(e.to_string()))
    }
}

/// Exactly the bytes the guest commits to the receipt journal.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GuestJournal {
    /// `wickra-proof` canonical hash of the backtest report (hex, 64 chars).
    pub report_hash: String,
    /// Dataset commitment, echoed and re-verified inside the guest.
    pub dataset_commitment: String,
    /// Sharpe ratio, rounded with [`round_to`] to 1e-8.
    pub sharpe: f64,
    /// Net PnL, rounded with [`round_to`] to 1e-8.
    pub pnl: f64,
    /// Number of closed trades.
    pub n_trades: u64,
}

/// The host-facing public outputs: the guest journal plus the pinned image id.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PublicOutputs {
    /// `wickra-proof` canonical hash of the backtest report (hex, 64 chars) —
    /// the same hash every language binding produces natively.
    pub report_hash: String,
    /// Dataset commitment the proof is bound to (hex, 64 chars).
    pub dataset_commitment: String,
    /// Image id of the guest that produced this receipt (hex) — proves *which*
    /// program ran.
    pub guest_id: String,
    /// Sharpe ratio (rounded to 1e-8).
    pub sharpe: f64,
    /// Net PnL (rounded to 1e-8).
    pub pnl: f64,
    /// Number of closed trades.
    pub n_trades: u64,
}

impl PublicOutputs {
    /// Combine a decoded [`GuestJournal`] with the host's pinned `guest_id`.
    #[must_use]
    pub fn from_journal(journal: GuestJournal, guest_id: String) -> Self {
        Self {
            report_hash: journal.report_hash,
            dataset_commitment: journal.dataset_commitment,
            guest_id,
            sharpe: journal.sharpe,
            pnl: journal.pnl,
            n_trades: journal.n_trades,
        }
    }
}

/// Options controlling how a proof is produced.
#[derive(Clone, Debug, Default)]
pub struct ProveOptions {
    /// When true, request a fast dev-mode receipt (NOT sound — for tests only).
    /// Production proving is driven by the prover backend / `RISC0_DEV_MODE`.
    pub dev_mode: bool,
}

/// A persisted proof: the receipt, its host-augmented public outputs and the
/// producing crate version. This is the stable on-disk boundary.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ZkProof {
    /// The risc0 receipt — the cryptographic object a verifier checks.
    pub receipt: risc0_zkvm::Receipt,
    /// Public outputs decoded from the receipt journal, plus the image id.
    pub journal: PublicOutputs,
    /// Version of `wickra-zk-host` that produced this proof.
    pub version: String,
}

impl ZkProof {
    /// Parse a `ZkProof` from JSON.
    ///
    /// # Errors
    /// Returns [`Error::Parse`] on malformed JSON.
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| Error::Parse(e.to_string()))
    }

    /// Serialise this proof to JSON.
    ///
    /// # Errors
    /// Returns [`Error::Parse`] if serialisation fails.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| Error::Parse(e.to_string()))
    }
}
