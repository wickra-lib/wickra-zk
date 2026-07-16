//! Verify a proof against the pinned guest image id.

use wickra_zk_methods::WICKRA_ZK_GUEST_ID;

use crate::error::{Error, Result};
use crate::model::{GuestJournal, PublicOutputs, ZkProof};

/// Verify `proof` and return its public outputs.
///
/// Checks the receipt against `WICKRA_ZK_GUEST_ID` — a receipt for any other
/// program fails here — then decodes the journal and, defence-in-depth,
/// confirms the persisted `guest_id` matches the pinned image id.
///
/// # Errors
/// Returns [`Error::Verify`] if the receipt is invalid, the image id is wrong,
/// the journal cannot be decoded, or the persisted `guest_id` disagrees.
pub fn verify(proof: &ZkProof) -> Result<PublicOutputs> {
    proof
        .receipt
        .verify(WICKRA_ZK_GUEST_ID)
        .map_err(|e| Error::Verify(e.to_string()))?;

    let journal: GuestJournal = proof
        .receipt
        .journal
        .decode()
        .map_err(|e| Error::Verify(e.to_string()))?;

    let outputs = PublicOutputs::from_journal(journal, crate::guest_id());
    if proof.journal.guest_id != outputs.guest_id {
        return Err(Error::Verify("guest id mismatch".to_string()));
    }
    Ok(outputs)
}
