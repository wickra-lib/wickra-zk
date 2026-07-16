//! Generated guest artifacts for wickra-zk.
//!
//! This crate exposes the compiled guest program to the host. The constants
//! come from `build.rs` via `risc0_build::embed_methods()`:
//!
//! - [`WICKRA_ZK_GUEST_ELF`] — the guest ELF bytes the prover executes.
//! - [`WICKRA_ZK_GUEST_ID`] — the image ID the verifier checks a receipt
//!   against; it is the cryptographic commitment to *which* program ran.
//!
//! The host crate (`wickra-zk-host`) proves with the ELF and verifies against
//! the ID, so a receipt is only accepted for this exact guest.

include!(concat!(env!("OUT_DIR"), "/methods.rs"));
