# Examples

One runnable example per language, all proving the same inputs: the strategy
in `specs/momentum.json` over the candles in `data/BTCUSDT.csv`. Every one
prints the same four lines before the journal:

```text
wickra-zk 0.1.0
guest_id: <the image id the host pins -- see the release notes>
report_hash: 1ec948078c8a02f92a0e7d28f9845a84f89aa1e1d0f87449c01389dd16dd6a53
verify: valid
```

The `report_hash` is the canonical `wickra-proof` hash of the backtest report
the guest computed. It is the same in every language because the same guest
ran; the Examples CI job holds each example to the value above, so the
document and the code cannot drift apart. The `guest_id` changes whenever the
guest program changes and is published with each release.

## Layout

| Path | Contents |
|------|----------|
| `rust/` | In-process: build a `ZkSpec`, `prove`, `verify`, print the journal. |
| `python/`, `node/`, `go/`, `java/`, `csharp/`, `r/` | Through each binding's command envelope; the dataset commitment is left to the host. |
| `c/` | The C and C++ examples and the C ABI golden test, built with CMake. |
| `specs/` | Valid `StrategySpec`s (`momentum`, `mean_reversion`, `crossover`). |
| `data/` | Deterministic OHLCV series (`BTCUSDT.csv`, `ETHUSDT.csv`), `ts,open,high,low,close,volume`. |

## Run (dev-mode)

Dev-mode produces a fast, **unsound** receipt -- right for a demo, never for a
real attestation. Without `RISC0_DEV_MODE=1` every example runs the real
prover, in-process, and takes minutes.

```bash
RISC0_DEV_MODE=1 cargo run --manifest-path examples/rust/Cargo.toml
RISC0_DEV_MODE=1 python examples/python/prove.py          # pip install wickra-zk
RISC0_DEV_MODE=1 node examples/node/prove.js              # cd examples/node && npm install
RISC0_DEV_MODE=1 go run ./examples/go                     # cgo; the C ABI library on the loader path
RISC0_DEV_MODE=1 dotnet run --project examples/csharp/Prove
RISC0_DEV_MODE=1 Rscript examples/r/prove.R
```

Each file's header names what it needs built first (the C ABI library for
Go, C#, Java and R; the Java classes for Java). The C examples:

```bash
cargo build --release -p wickra-zk-c
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

## From the CLI

Equivalently, with the `wickra-zk` binary:

```bash
wickra-zk prove --spec examples/specs/momentum.json --data examples/data/BTCUSDT.csv --out proof.json --dev
wickra-zk verify --proof proof.json
```

> Dev-mode (`--dev` / `RISC0_DEV_MODE=1`) is for demos only. Verifiers must
> reject dev-mode receipts in production -- see [`../SECURITY.md`](../SECURITY.md).
