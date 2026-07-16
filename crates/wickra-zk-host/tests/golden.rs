//! Golden determinism chain: native hash == guest journal == blessed expected.

mod common;

use common::{case, dev_mode, load_candles, load_expected, load_strategy, CASES};
use wickra_zk_host::{native_report_hash, prove, verify, ProveOptions};

/// (a) The native path reproduces the blessed `report_hash`.
#[test]
fn native_hash_matches_expected() {
    for (name, data) in CASES {
        let strategy = load_strategy(name);
        let candles = load_candles(data);
        let expected = load_expected(name);
        let hash = native_report_hash(&strategy, &candles).unwrap();
        assert_eq!(hash, expected.report_hash, "native hash drift for {name}");
    }
}

/// (b)+(c) The dev-mode guest reproduces the native hash and the metrics —
/// the determinism chain is closed end to end.
#[test]
fn dev_mode_chain_closes() {
    dev_mode();
    for (name, data) in CASES {
        let (spec, candles) = case(name, data);
        let expected = load_expected(name);
        let proof = prove(&spec, &candles, ProveOptions { dev_mode: true }).unwrap();
        let out = verify(&proof).unwrap();
        assert_eq!(out.report_hash, expected.report_hash, "guest hash != native for {name}");
        assert!((out.sharpe - expected.sharpe).abs() < 1e-8, "sharpe drift for {name}");
        assert!((out.pnl - expected.pnl).abs() < 1e-8, "pnl drift for {name}");
        assert_eq!(out.n_trades, expected.n_trades, "n_trades drift for {name}");
    }
}
