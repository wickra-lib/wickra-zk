//! JSON command envelope shared by every binding (see docs/ZK.md, "The
//! command envelope").
//!
//! A single string-in/string-out entry point so C, Python, Node, … all drive
//! the host over the same boundary:
//!
//! - `{"cmd":"prove","spec":{"strategy":<StrategySpec>,"dataset_commitment"?:<hex>},"candles":[<Candle>,…]}`
//!   → a `ZkProof` JSON. The commitment is optional: a caller that holds the
//!   candles need not carry a canonical hasher, the host computes it the same
//!   way the guest recomputes it. A caller that states one is held to it.
//! - `{"cmd":"commit","candles":[<Candle>,…]}` → `{"dataset_commitment":<hex>}`,
//!   so a verifier who holds the data can check what a proof is bound to.
//! - `{"cmd":"verify","proof":<ZkProof>}` → the `PublicOutputs` JSON.
//! - `{"cmd":"version"}` → `{"version":"<x.y.z>","guest_id":"<hex>"}`.

use serde::Deserialize;
use wickra_backtest::{Candle, StrategySpec};

use crate::error::{Error, Result};
use crate::model::{ProveOptions, ZkProof, ZkSpec};

/// The `spec` of a `prove` command: a [`ZkSpec`] whose commitment may be left
/// to the host.
#[derive(Deserialize)]
struct ProveSpec {
    strategy: StrategySpec,
    #[serde(default)]
    dataset_commitment: Option<String>,
}

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
enum Command {
    Prove {
        spec: ProveSpec,
        candles: Vec<Candle>,
    },
    Commit {
        candles: Vec<Candle>,
    },
    Verify {
        proof: ZkProof,
    },
    Version,
}

fn to_json<T: serde::Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(|e| Error::Parse(e.to_string()))
}

/// Dispatch a JSON command envelope and return the JSON response.
///
/// # Errors
/// Returns [`Error::Parse`] for a malformed envelope and propagates
/// [`crate::prove`] / [`crate::verify`] / [`crate::commit_dataset`] errors.
pub fn command_json(cmd_json: &str) -> Result<String> {
    let cmd: Command = serde_json::from_str(cmd_json).map_err(|e| Error::Parse(e.to_string()))?;
    match cmd {
        Command::Prove { spec, candles } => {
            let dataset_commitment = match spec.dataset_commitment {
                Some(commitment) => commitment,
                None => crate::commit_dataset(&candles)?,
            };
            let spec = ZkSpec {
                strategy: spec.strategy,
                dataset_commitment,
            };
            let proof = crate::prove(&spec, &candles, ProveOptions::default())?;
            proof.to_json()
        }
        Command::Commit { candles } => {
            let dataset_commitment = crate::commit_dataset(&candles)?;
            to_json(&serde_json::json!({ "dataset_commitment": dataset_commitment }))
        }
        Command::Verify { proof } => to_json(&crate::verify(&proof)?),
        Command::Version => to_json(&serde_json::json!({
            "version": crate::version(),
            "guest_id": crate::guest_id(),
        })),
    }
}
