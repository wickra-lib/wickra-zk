//! Real (non dev-mode) proving. Slow and memory-hungry — `#[ignore]`d by
//! default and run in the nightly `prove.yml` workflow.

mod common;

use common::{case, cases, load_expected};
use wickra_zk_host::{prove, verify, ProveOptions};

#[test]
#[ignore = "real proving is slow; run via `cargo test -- --ignored` in prove.yml"]
fn prod_prove_and_verify() {
    for (name, data) in &cases() {
        let (spec, candles) = case(name, data);
        let expected = load_expected(name);
        // No dev-mode: a real, sound receipt.
        let proof = prove(&spec, &candles, ProveOptions { dev_mode: false }).unwrap();
        let out = verify(&proof).unwrap();
        // The proof does not change the public outputs — same journal as dev-mode.
        assert_eq!(
            out.report_hash, expected.report_hash,
            "prod hash drift for {name}"
        );
        assert_eq!(out.n_trades, expected.n_trades);
    }
}
