//! Produce a proof by executing the guest inside the zkVM.

use risc0_zkvm::{default_prover, ExecutorEnv};
use wickra_backtest::Candle;
use wickra_zk_methods::WICKRA_ZK_GUEST_ELF;

use crate::error::{Error, Result};
use crate::model::{GuestJournal, ProveOptions, PublicOutputs, ZkProof, ZkSpec};

/// Prove an honest run of the backtest over `candles` under `spec.strategy`.
///
/// The strategy, candles and dataset commitment are written to the executor as
/// private inputs; the guest binds the data to the commitment, runs the engine,
/// hashes the report and commits [`GuestJournal`]. The returned [`ZkProof`]
/// carries the receipt plus the host-augmented public outputs.
///
/// # Errors
/// Returns [`Error::Prove`] if the executor cannot be built, the guest panics
/// (e.g. commitment mismatch), or the journal cannot be decoded.
pub fn prove(spec: &ZkSpec, candles: &[Candle], _opts: ProveOptions) -> Result<ZkProof> {
    // The strategy crosses as JSON text: risc0's serde is a word stream with
    // no field names or tags, and `StrategySpec` relies on JSON semantics
    // (`skip_serializing_if` on an `Option`, an `untagged` enum), so written as
    // a value it does not read back. See the guest's `main.rs`.
    let strategy_json =
        serde_json::to_string(&spec.strategy).map_err(|e| Error::Prove(e.to_string()))?;
    let env = ExecutorEnv::builder()
        .write(&strategy_json)
        .map_err(|e| Error::Prove(e.to_string()))?
        .write(&candles)
        .map_err(|e| Error::Prove(e.to_string()))?
        .write(&spec.dataset_commitment)
        .map_err(|e| Error::Prove(e.to_string()))?
        .build()
        .map_err(|e| Error::Prove(e.to_string()))?;

    let prove_info = default_prover()
        .prove(env, WICKRA_ZK_GUEST_ELF)
        .map_err(|e| Error::Prove(e.to_string()))?;
    let receipt = prove_info.receipt;

    let journal: GuestJournal = receipt
        .journal
        .decode()
        .map_err(|e| Error::Prove(e.to_string()))?;

    let outputs = PublicOutputs::from_journal(journal, crate::guest_id());
    Ok(ZkProof {
        receipt,
        journal: outputs,
        version: crate::version().to_string(),
    })
}
