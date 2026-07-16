//! JSON command envelope shared by every binding (see docs/ZK.md §6.8).
//!
//! A single string-in/string-out entry point so C, Python, Node, … all drive
//! the host over the same boundary:
//!
//! - `{"cmd":"prove","spec":<ZkSpec>,"candles":[<Candle>,…]}` → a `ZkProof` JSON
//! - `{"cmd":"verify","proof":<ZkProof>}` → a `PublicOutputs` JSON
//! - `{"cmd":"version"}` → `{"version":"<x.y.z>"}`

use serde::Deserialize;
use wickra_backtest::Candle;

use crate::error::{Error, Result};
use crate::model::{ProveOptions, ZkProof, ZkSpec};

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
enum Command {
    Prove { spec: ZkSpec, candles: Vec<Candle> },
    Verify { proof: ZkProof },
    Version,
}

/// Dispatch a JSON command envelope and return the JSON response.
///
/// # Errors
/// Returns [`Error::Parse`] for a malformed envelope and propagates
/// [`crate::prove`] / [`crate::verify`] errors.
pub fn command_json(cmd_json: &str) -> Result<String> {
    let cmd: Command = serde_json::from_str(cmd_json).map_err(|e| Error::Parse(e.to_string()))?;
    match cmd {
        Command::Prove { spec, candles } => {
            let proof = crate::prove(&spec, &candles, ProveOptions::default())?;
            proof.to_json()
        }
        Command::Verify { proof } => {
            let out = crate::verify(&proof)?;
            serde_json::to_string(&out).map_err(|e| Error::Parse(e.to_string()))
        }
        Command::Version => Ok(format!(r#"{{"version":"{}"}}"#, crate::version())),
    }
}
