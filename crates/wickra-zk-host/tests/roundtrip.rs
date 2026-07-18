//! Dev-mode prove/verify round-trip and tamper detection.

mod common;

use common::{case, dev_mode};
use wickra_zk_host::{prove, verify, ProveOptions};

#[test]
fn prove_then_verify_roundtrips_in_dev_mode() {
    dev_mode();
    let (spec, candles) = case("momentum", "sym-01");
    let proof = prove(&spec, &candles, ProveOptions { dev_mode: true }).unwrap();

    let out = verify(&proof).unwrap();
    assert_eq!(out.report_hash, proof.journal.report_hash);
    assert_eq!(out.dataset_commitment, spec.dataset_commitment);
    assert!(!out.guest_id.is_empty());
}

#[test]
fn tampered_guest_id_fails_verify() {
    dev_mode();
    let (spec, candles) = case("momentum", "sym-01");
    let mut proof = prove(&spec, &candles, ProveOptions { dev_mode: true }).unwrap();

    // The persisted journal claims a different program than the receipt binds.
    proof.journal.guest_id = "0".repeat(64);
    assert!(verify(&proof).is_err());
}
