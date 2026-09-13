//! The JSON command envelope every binding drives, end to end in dev-mode.
//!
//! The bindings are thin: each hands a string to `command_json` and hands the
//! answer back. What they can and cannot do is therefore decided here, and the
//! one thing a binding cannot do on its own is canonically hash the candles --
//! so `prove` must work without a `dataset_commitment`, and `commit` must
//! produce the one the guest will hold the proof to.

mod common;

use common::{case, dev_mode, load_candles, load_strategy};
use serde_json::{json, Value};
use wickra_zk_host::{command_json, guest_id, version, Error};

fn run(cmd: &Value) -> Value {
    serde_json::from_str(&command_json(&cmd.to_string()).unwrap()).unwrap()
}

#[test]
fn version_names_the_crate_and_the_guest() {
    let response = run(&json!({ "cmd": "version" }));
    assert_eq!(response["version"], version());
    assert_eq!(response["guest_id"], guest_id());
}

#[test]
fn commit_is_the_commitment_the_guest_holds_the_proof_to() {
    let (spec, candles) = case("momentum", "sym-01");
    let response = run(&json!({ "cmd": "commit", "candles": candles }));
    assert_eq!(response["dataset_commitment"], spec.dataset_commitment);
}

#[test]
fn prove_without_a_commitment_binds_the_proof_to_the_candles() {
    dev_mode();
    let strategy = load_strategy("momentum");
    let candles = load_candles("sym-01");
    let (spec, _) = case("momentum", "sym-01");

    let proof = run(&json!({
        "cmd": "prove",
        "spec": { "strategy": strategy },
        "candles": candles,
    }));
    assert_eq!(
        proof["journal"]["dataset_commitment"],
        spec.dataset_commitment
    );
    assert_eq!(proof["journal"]["guest_id"], guest_id());

    let outputs = run(&json!({ "cmd": "verify", "proof": proof }));
    assert_eq!(outputs, proof["journal"]);
}

#[test]
fn a_stated_commitment_is_held_to() {
    dev_mode();
    let strategy = load_strategy("momentum");
    let candles = load_candles("sym-01");
    let cmd = json!({
        "cmd": "prove",
        "spec": { "strategy": strategy, "dataset_commitment": "0".repeat(64) },
        "candles": candles,
    });
    let err = command_json(&cmd.to_string()).unwrap_err();
    assert!(matches!(err, Error::Commitment), "{err}");
}

#[test]
fn a_proof_file_whose_journal_disagrees_with_its_receipt_does_not_verify() {
    dev_mode();
    let strategy = load_strategy("momentum");
    let candles = load_candles("sym-01");
    let mut proof = run(&json!({
        "cmd": "prove",
        "spec": { "strategy": strategy },
        "candles": candles,
    }));

    // The receipt is untouched; only the human-readable copy beside it lies.
    proof["journal"]["report_hash"] = Value::String("f".repeat(64));
    let err = command_json(&json!({ "cmd": "verify", "proof": proof }).to_string()).unwrap_err();
    assert!(matches!(err, Error::Verify(_)), "{err}");
}
