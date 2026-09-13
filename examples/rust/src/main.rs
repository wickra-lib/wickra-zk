//! Prove a backtest in zero knowledge from Rust, then verify the proof.
//!
//! Reads the shared example inputs (`../specs/momentum.json` and
//! `../data/BTCUSDT.csv`), proves them in-process, verifies the receipt and
//! prints the public journal. With `RISC0_DEV_MODE=1` the receipt is a fast,
//! unsound placeholder; without it the real prover runs and this takes
//! minutes.
//!
//! ```text
//! RISC0_DEV_MODE=1 cargo run
//! ```

use std::fs;
use std::path::PathBuf;

use serde_json::json;
use wickra_zk_host::{commit_dataset, prove, verify, Candle, ProveOptions, StrategySpec, ZkSpec};

fn examples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn load_candles(path: PathBuf) -> Vec<Candle> {
    let content = fs::read_to_string(path).expect("read data csv");
    let mut candles = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        let cols: Vec<&str> = line.trim().split(',').map(str::trim).collect();
        if cols.len() != 6 || (idx == 0 && cols[0].parse::<i64>().is_err()) {
            continue;
        }
        let value = json!({
            "time": cols[0].parse::<i64>().unwrap(), "open": cols[1].parse::<f64>().unwrap(),
            "high": cols[2].parse::<f64>().unwrap(), "low": cols[3].parse::<f64>().unwrap(),
            "close": cols[4].parse::<f64>().unwrap(), "volume": cols[5].parse::<f64>().unwrap(),
        });
        candles.push(serde_json::from_value(value).unwrap());
    }
    candles
}

fn main() {
    // Private inputs -- never leave the guest.
    let strategy: StrategySpec = serde_json::from_str(
        &fs::read_to_string(examples_dir().join("specs/momentum.json")).unwrap(),
    )
    .unwrap();
    let candles = load_candles(examples_dir().join("data/BTCUSDT.csv"));

    // Bind the proof to this exact data without revealing it. (Over the
    // command envelope the host computes this when the spec omits it.)
    let spec = ZkSpec {
        strategy,
        dataset_commitment: commit_dataset(&candles).expect("candles canonicalise"),
    };

    // Prove, then verify the receipt and print the public journal.
    let proof = prove(&spec, &candles, ProveOptions { dev_mode: true }).expect("prove");
    let journal = verify(&proof).expect("verify");

    println!("wickra-zk {}", wickra_zk_host::version());
    println!("guest_id: {}", journal.guest_id);
    println!("report_hash: {}", journal.report_hash);
    println!("verify: {}", if journal == proof.journal { "valid" } else { "INVALID" });
    println!("{}", serde_json::to_string_pretty(&journal).unwrap());
}
