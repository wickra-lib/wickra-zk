# Benchmarks

Measured by the `wickra-zk-bench` Criterion suite (`crates/wickra-zk-bench/benches/prove.rs`)
on the `momentum` golden case -- a fast/slow SMA cross over `sym-01`, 48 OHLCV
bars -- on a GitHub-hosted `ubuntu-latest` runner (4 vCPU, CPU prover, no GPU).
Medians from the nightly `bench.yml` run of 2026-09-15, run
[35007405126](https://github.com/wickra-lib/wickra-zk/actions/runs/35007405126);
the whole job took 3 h 39 min, build included.

| Measurement | What it times | Samples | Median |
|-------------|---------------|---------|--------|
| `exec/momentum` | Dev-mode proving: the guest runs the backtest inside the zkVM and produces the journal, no real proof (`RISC0_DEV_MODE=1`). | 100 | **237.4 ms** |
| `prove/momentum` | Production proving: a real RISC Zero receipt for the same case, in-process on the CPU. | 10 | **1056.6 s** (17 min 37 s) |
| `verify/momentum` | Checking that receipt against the pinned guest id (`WICKRA_ZK_GUEST_ID`). | 100 | **19.09 ms** |

Proving dominates by four orders of magnitude over execution and five over
verification, which is the shape a zkVM proof is supposed to have: the
backtest itself costs a quarter of a second inside the guest, producing the
proof of it costs minutes of STARK arithmetic, and anyone can check the result
in twenty milliseconds without the data or the strategy.

## Reading the numbers

- **Execution** is what a caller pays on every `prove` call before proving
  starts, and what the dev-mode tests pay in full. It scales with the number
  of bars and the indicators the strategy names.
- **Proving** scales with the guest's cycle count in segments; a longer
  history or a heavier strategy adds segments and proving time roughly
  linearly. The CPU prover on a hosted runner is the floor, not the ceiling:
  a machine with more cores, or the CUDA / Metal backends risc0 ships, proves
  the same case several times faster. The receipt is the same bytes either
  way.
- **Verification** does not depend on the history length or the strategy;
  it is the cost of checking the receipt's STARK, and it is what a counterparty
  runs.

## Reproducing

```bash
# execution only, seconds:
cargo bench -p wickra-zk-bench
# execution, proving and verification, hours (one real proof per sample):
WICKRA_ZK_BENCH_PROD=1 cargo bench -p wickra-zk-bench
```

The prover needs clang (`CC=clang CXX=clang++`; the C++ kernels take minutes
with g++ and seconds with clang) and no Windows host. `bench.yml` runs the
production suite nightly under a six-hour budget: Criterion's minimum of ten
samples at seventeen minutes a proof, plus one warm-up proof and the receipt
the verify bench checks, is close to three and a half hours.
