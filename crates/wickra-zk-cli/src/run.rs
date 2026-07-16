//! Subcommand implementations.

use std::fs;
use std::path::Path;

use wickra_zk_host::{
    commit_dataset, guest_id, prove, verify, Candle, ProveOptions, StrategySpec, ZkProof, ZkSpec,
};

use crate::args::Command;

/// Run a command. Returns `Ok(true)` on success / `verified:true`, `Ok(false)`
/// when a verification is negative (so `main` can exit non-zero), and `Err` on
/// any operational failure.
pub fn run(command: Command) -> Result<bool, String> {
    match command {
        Command::Prove {
            spec,
            data,
            out,
            dev,
        } => {
            let zk_spec = load_spec(&spec, &data)?;
            let candles = load_candles(&data)?;
            let proof = prove(&zk_spec, &candles, ProveOptions { dev_mode: dev })
                .map_err(|e| e.to_string())?;
            let json = proof.to_json().map_err(|e| e.to_string())?;
            match out {
                Some(path) => {
                    fs::write(&path, json).map_err(|e| format!("write {}: {e}", path.display()))?;
                }
                None => println!("{json}"),
            }
            Ok(true)
        }
        Command::Verify { proof } => {
            let content =
                fs::read_to_string(&proof).map_err(|e| format!("read {}: {e}", proof.display()))?;
            let zk_proof = ZkProof::from_json(&content).map_err(|e| e.to_string())?;
            match verify(&zk_proof) {
                Ok(journal) => {
                    let journal = serde_json::to_value(&journal).map_err(|e| e.to_string())?;
                    println!(
                        "{}",
                        serde_json::json!({ "verified": true, "journal": journal })
                    );
                    Ok(true)
                }
                Err(e) => {
                    println!(
                        "{}",
                        serde_json::json!({ "verified": false, "error": e.to_string() })
                    );
                    Ok(false)
                }
            }
        }
        Command::Version => {
            println!(
                "{}",
                serde_json::json!({ "version": wickra_zk_host::version(), "guest_id": guest_id() })
            );
            Ok(true)
        }
    }
}

/// Load a `ZkSpec` from `spec_path`, computing the dataset commitment from the
/// candles when the spec omits it (a bare `StrategySpec` is also accepted).
fn load_spec(spec_path: &Path, data_path: &Path) -> Result<ZkSpec, String> {
    let content =
        fs::read_to_string(spec_path).map_err(|e| format!("read {}: {e}", spec_path.display()))?;

    // Already a complete ZkSpec?
    if let Ok(zk) = ZkSpec::from_json(&content) {
        return Ok(zk);
    }

    // Otherwise: either { "strategy": <spec>, ["dataset_commitment": ...] } or a
    // bare StrategySpec. Compute the commitment from the data when missing.
    let value: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("parse {}: {e}", spec_path.display()))?;
    let (strategy_value, commitment) = match value.get("strategy") {
        Some(strategy) => (
            strategy.clone(),
            value
                .get("dataset_commitment")
                .and_then(|c| c.as_str())
                .map(ToString::to_string),
        ),
        None => (value, None),
    };
    let strategy: StrategySpec =
        serde_json::from_value(strategy_value).map_err(|e| format!("strategy: {e}"))?;
    let dataset_commitment = match commitment {
        Some(c) => c,
        None => commit_dataset(&load_candles(data_path)?),
    };
    Ok(ZkSpec {
        strategy,
        dataset_commitment,
    })
}

/// Parse a `time,open,high,low,close,volume` CSV into candles. A non-numeric
/// first line is treated as a header and skipped.
fn load_candles(path: &Path) -> Result<Vec<Candle>, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let mut candles = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split(',').map(str::trim).collect();
        if cols.len() != 6 {
            return Err(format!("{}: line {}: expected 6 columns", path.display(), idx + 1));
        }
        if idx == 0 && cols[0].parse::<i64>().is_err() {
            continue; // header row
        }
        let time: i64 = cols[0].parse().map_err(|_| format!("line {}: bad time", idx + 1))?;
        let mut num = [0f64; 5];
        for (slot, raw) in num.iter_mut().zip(&cols[1..]) {
            *slot = raw
                .parse()
                .map_err(|_| format!("line {}: bad number '{raw}'", idx + 1))?;
        }
        let value = serde_json::json!({
            "time": time,
            "open": num[0],
            "high": num[1],
            "low": num[2],
            "close": num[3],
            "volume": num[4],
        });
        let candle: Candle =
            serde_json::from_value(value).map_err(|e| format!("line {}: {e}", idx + 1))?;
        candles.push(candle);
    }
    if candles.is_empty() {
        return Err(format!("no candles in {}", path.display()));
    }
    Ok(candles)
}
