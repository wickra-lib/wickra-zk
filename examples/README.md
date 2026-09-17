# Wickra ZK examples

One runnable example per language, all proving the same inputs: the strategy
in `specs/momentum.json` over the candles in `data/BTCUSDT.csv`. Every one
prints the same four lines before the journal:

## What every example prints

One runnable example per language, all proving the same inputs: the strategy
in `specs/momentum.json` over the candles in `data/BTCUSDT.csv`. Every one
prints the same four lines before the journal:

```text
wickra-zk 0.1.1
guest_id: <the image id the host pins -- see the release notes>
report_hash: 1ec948078c8a02f92a0e7d28f9845a84f89aa1e1d0f87449c01389dd16dd6a53
verify: valid
```

## Rust — `examples/rust/`

As the CI examples job runs it, from the repository root:

```bash
cargo run -q --manifest-path examples/rust/Cargo.toml
```

| Example | What it does |
| --- | --- |
| `src/main.rs` | Prove a backtest in zero knowledge from Rust, then verify the proof. |

## C / C++ — `examples/c/`

Build the library first (`cargo build -p wickra-zk-c --release`), then build and run
the examples via CMake, as the CI C ABI job does:

```bash
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

| Example | What it does |
| --- | --- |
| `prove.c` | A minimal C example: prove a (strategy, candles) pair through the wickra-zk |
| `prove.cpp` | A minimal C++ example: prove a (strategy, candles) pair through the wickra-zk C ABI, print the public journal, then verify the proof and assert it holds. |

## C# — `examples/csharp/`

As the CI examples job runs it, from the repository root:

```bash
dotnet run --project examples/csharp/Prove
```

| Example | What it does |
| --- | --- |
| `Prove/Program.cs` | Prove a backtest in zero knowledge from .NET, then verify the proof. |

## Go — `examples/go/`

As the CI examples job runs it, from the repository root:

```bash
cd examples/go && go run .
```

| Example | What it does |
| --- | --- |
| `prove.go` | Prove a backtest in zero knowledge from Go, then verify the proof. |

## R — `examples/r/`

As the CI examples job runs it, from the repository root:

```bash
R CMD INSTALL bindings/r
Rscript examples/r/prove.R
```

| Example | What it does |
| --- | --- |
| `prove.R` | Prove a backtest in zero knowledge from R, then verify the proof. |

## Java — `examples/java/`

As the CI examples job runs it, from the repository root:

```bash
mvn -f bindings/java/pom.xml -q package -DskipTests
javac -cp bindings/java/target/classes examples/java/Prove.java -d examples/java/out
java --enable-native-access=ALL-UNNAMED  -Dnative.lib.dir="$PWD/target/release"  -cp "bindings/java/target/classes:examples/java/out" Prove examples
```

| Example | What it does |
| --- | --- |
| `Prove.java` | Prove a backtest in zero knowledge from Java, then verify the proof. |

## Python — `examples/python/`

As the CI examples job runs it, from the repository root:

```bash
python -m pip install --require-hashes -r .github/requirements/ci-dev-py3.txt
( cd bindings/python && maturin build --release --out dist )
python -m pip install --no-index --find-links bindings/python/dist wickra-zk
python examples/python/prove.py
```

| Example | What it does |
| --- | --- |
| `prove.py` | Prove a backtest in zero knowledge from Python, then verify the proof. |

## Node.js — `examples/node/`

As the CI examples job runs it, from the repository root:

```bash
( cd bindings/node && npm install --no-audit --no-fund && npx napi build --platform --release )
( cd examples/node && npm install --no-audit --no-fund )
node examples/node/prove.js
```

| Example | What it does |
| --- | --- |
| `prove.js` | Prove a backtest in zero knowledge from Node.js, then verify the proof. |

## WASM — `examples/wasm/`

Build the WASM package, serve the repository root, and open the page in a browser;
the module script inside it is what runs (CI parses it with `node --check`):

```bash
wasm-pack build bindings/wasm --target web
python -m http.server 8000     # then open http://localhost:8000/examples/wasm/
```

| Example | What it does |
| --- | --- |
| `verify.html` | A runnable example against this binding. |

## Example datasets

The examples read from [`examples/data/`](data/): `BTCUSDT.csv`, `ETHUSDT.csv`. The
cross-language golden fixtures, which every binding is checked against byte for
byte, live in [`../golden/`](../golden).

## Layout

| Path | Contents |
|------|----------|
| `rust/` | In-process: build a `ZkSpec`, `prove`, `verify`, print the journal. |
| `python/`, `node/`, `go/`, `java/`, `csharp/`, `r/` | Through each binding's command envelope; the dataset commitment is left to the host. |
| `c/` | The C and C++ examples and the C ABI golden test, built with CMake. |
| `wasm/` | The verifying side in a browser: fetches a real golden receipt and decodes its journal -- see [`wasm/README.md`](wasm/README.md). |
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
