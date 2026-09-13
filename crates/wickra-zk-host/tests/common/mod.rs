//! Shared golden loaders for the integration tests.
#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::json;
use wickra_zk_host::{commit_dataset, Candle, StrategySpec, ZkSpec};

/// The golden cases and the dataset each runs over, from `golden/cases.json`
/// -- the one mapping every binding's golden test reads too.
pub fn cases() -> Vec<(String, String)> {
    let text = fs::read_to_string(golden_dir().join("cases.json")).unwrap();
    let map: std::collections::BTreeMap<String, String> = serde_json::from_str(&text).unwrap();
    map.into_iter().collect()
}

/// Blessed expected outputs for one case.
#[derive(Deserialize)]
pub struct Expected {
    pub report_hash: String,
    pub sharpe: f64,
    pub pnl: f64,
    pub n_trades: u64,
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../golden")
}

pub fn load_strategy(name: &str) -> StrategySpec {
    let path = golden_dir().join("specs").join(format!("{name}.json"));
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

pub fn load_expected(name: &str) -> Expected {
    let path = golden_dir().join("expected").join(format!("{name}.json"));
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

pub fn load_candles(data: &str) -> Vec<Candle> {
    let path = golden_dir().join("data").join(format!("{data}.csv"));
    let content = fs::read_to_string(path).unwrap();
    let mut candles = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split(',').map(str::trim).collect();
        if idx == 0 && cols[0].parse::<i64>().is_err() {
            continue;
        }
        let value = json!({
            "time": cols[0].parse::<i64>().unwrap(),
            "open": cols[1].parse::<f64>().unwrap(),
            "high": cols[2].parse::<f64>().unwrap(),
            "low": cols[3].parse::<f64>().unwrap(),
            "close": cols[4].parse::<f64>().unwrap(),
            "volume": cols[5].parse::<f64>().unwrap(),
        });
        candles.push(serde_json::from_value(value).unwrap());
    }
    candles
}

/// Build the `ZkSpec` (with computed commitment) and candles for a case.
pub fn case(name: &str, data: &str) -> (ZkSpec, Vec<Candle>) {
    let strategy = load_strategy(name);
    let candles = load_candles(data);
    let dataset_commitment = commit_dataset(&candles).expect("candles canonicalise");
    (
        ZkSpec {
            strategy,
            dataset_commitment,
        },
        candles,
    )
}

/// Set dev-mode for a fast (unsound) receipt in tests.
pub fn dev_mode() {
    // SAFETY: single-threaded test setup; risc0 reads this at prove time.
    std::env::set_var("RISC0_DEV_MODE", "1");
}
