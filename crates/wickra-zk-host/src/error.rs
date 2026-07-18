//! Host error type.

/// Errors returned by the prover, verifier and command dispatcher.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A JSON input (spec, proof or command envelope) could not be parsed.
    #[error("parse: {0}")]
    Parse(String),
    /// Proving failed (executor setup, guest panic, or prover backend error).
    #[error("prove: {0}")]
    Prove(String),
    /// Verification failed (bad receipt, wrong image id, or journal mismatch).
    #[error("verify: {0}")]
    Verify(String),
    /// The dataset commitment did not match the candles fed to the guest.
    #[error("commitment mismatch")]
    Commitment,
    /// A dataset/candle input was invalid.
    #[error("data: {0}")]
    Data(String),
}

/// Convenience alias.
pub type Result<T> = core::result::Result<T, Error>;
