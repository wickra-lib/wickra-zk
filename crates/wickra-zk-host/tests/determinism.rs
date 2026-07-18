//! Proving the same inputs twice yields an identical journal.

mod common;

use common::{case, dev_mode};
use wickra_zk_host::{prove, ProveOptions};

#[test]
fn prove_is_deterministic() {
    dev_mode();
    let (spec, candles) = case("crossover", "sym-03");
    let first = prove(&spec, &candles, ProveOptions { dev_mode: true }).unwrap();
    let second = prove(&spec, &candles, ProveOptions { dev_mode: true }).unwrap();
    assert_eq!(first.journal, second.journal);
}
