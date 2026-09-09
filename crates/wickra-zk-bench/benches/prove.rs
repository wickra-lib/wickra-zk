//! Criterion benchmarks for wickra-zk.
//!
//! - `exec/<case>` — dev-mode proving: the cost of running the guest without a
//!   real proof (dominated by the backtest inside the zkVM).
//! - `prove/<case>` — real proving, gated behind `WICKRA_ZK_BENCH_PROD=1`
//!   because it is slow and memory-hungry; run in the nightly `bench.yml`.

use std::fs;
use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, Criterion};
use serde_json::json;
use wickra_zk_host::{commit_dataset, prove, Candle, ProveOptions, StrategySpec, ZkSpec};

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../golden")
}

fn load_case(name: &str, data: &str) -> (ZkSpec, Vec<Candle>) {
    let strategy: StrategySpec = serde_json::from_str(
        &fs::read_to_string(golden_dir().join("specs").join(format!("{name}.json"))).unwrap(),
    )
    .unwrap();
    let content =
        fs::read_to_string(golden_dir().join("data").join(format!("{data}.csv"))).unwrap();
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
    let dataset_commitment = commit_dataset(&candles).expect("candles canonicalise");
    (
        ZkSpec {
            strategy,
            dataset_commitment,
        },
        candles,
    )
}

fn bench_prove(c: &mut Criterion) {
    std::env::set_var("RISC0_DEV_MODE", "1");
    let (spec, candles) = load_case("momentum", "sym-01");

    c.bench_function("exec/momentum", |b| {
        b.iter(|| prove(&spec, &candles, ProveOptions { dev_mode: true }).unwrap());
    });

    if std::env::var("WICKRA_ZK_BENCH_PROD").as_deref() == Ok("1") {
        std::env::remove_var("RISC0_DEV_MODE");
        let mut group = c.benchmark_group("prove");
        group.sample_size(10);
        group.bench_function("momentum", |b| {
            b.iter(|| prove(&spec, &candles, ProveOptions { dev_mode: false }).unwrap());
        });
        group.finish();
    }
}

criterion_group!(benches, bench_prove);
criterion_main!(benches);
