//! A real receipt, verified for real, on every push.
//!
//! `golden/proofs/<case>.json` is a proof the real prover produced for that
//! golden case -- a succinct receipt, not a dev-mode placeholder. Verifying it
//! exercises the sound path (`Receipt::verify` against the pinned image id)
//! without proving, so a change that breaks verification of a genuine receipt
//! is caught here rather than by the nightly prover. The decoded journal must
//! be the blessed one, and the file's own journal copy must agree with it.
//!
//! The image id is the guest's: a receipt proves *that* program ran. When the
//! guest changes, its id changes, and this test fails until the proof is
//! regenerated for the new guest -- which is the point: a committed proof for
//! a guest that no longer exists is a proof of nothing.

mod common;

use std::fs;
use std::path::PathBuf;

use common::{case, load_expected};
use wickra_zk_host::{guest_id, verify, ZkProof};

fn proofs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../golden/proofs")
}

fn real_proofs() -> Vec<(String, ZkProof)> {
    let mut out = Vec::new();
    for entry in fs::read_dir(proofs_dir()).expect("golden/proofs exists") {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "json") {
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            let proof = ZkProof::from_json(&fs::read_to_string(&path).unwrap()).unwrap();
            out.push((name, proof));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    assert!(
        !out.is_empty(),
        "golden/proofs holds at least one real receipt"
    );
    out
}

#[test]
fn the_committed_real_receipts_verify_to_the_blessed_journal() {
    for (name, proof) in real_proofs() {
        let expected = load_expected(&name);
        let out = verify(&proof).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(out.report_hash, expected.report_hash, "{name}");
        assert_eq!(out.n_trades, expected.n_trades, "{name}");
        assert!((out.sharpe - expected.sharpe).abs() < 1e-8, "{name}");
        assert!((out.pnl - expected.pnl).abs() < 1e-8, "{name}");
        assert_eq!(
            out.guest_id,
            guest_id(),
            "{name}: the proof is for this guest"
        );
        assert_eq!(out, proof.journal, "{name}: the file's journal copy agrees");
    }
}

#[test]
fn the_committed_real_receipts_are_bound_to_the_golden_candles() {
    let cases = common::cases();
    for (name, proof) in real_proofs() {
        let dataset = &cases.iter().find(|(c, _)| *c == name).unwrap().1;
        let (spec, _) = case(&name, dataset);
        assert_eq!(
            proof.journal.dataset_commitment, spec.dataset_commitment,
            "{name}"
        );
    }
}

#[test]
fn a_real_receipt_with_a_lying_journal_copy_is_refused() {
    for (name, mut proof) in real_proofs() {
        proof.journal.report_hash = "f".repeat(64);
        assert!(verify(&proof).is_err(), "{name}");
    }
}
