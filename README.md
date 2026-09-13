<p align="center">
  <img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp" alt="Wickra" width="100%">
</p>

# wickra-zk

[![Built on Wickra](https://img.shields.io/badge/built%20on-wickra-3b82f6)](https://github.com/wickra-lib/wickra)
[![Status](https://img.shields.io/badge/status-pre--release-orange)](https://github.com/wickra-lib/wickra-zk)
[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/ci.svg)](https://github.com/wickra-lib/wickra-zk/actions/workflows/ci.yml)
[![CodeQL](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/codeql.svg)](https://github.com/wickra-lib/wickra-zk/actions/workflows/codeql.yml)
[![GitHub release](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/release.svg)](https://github.com/wickra-lib/wickra-zk/releases/latest)
[![crates.io](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/crates.svg)](https://crates.io/crates/wickra-zk)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/license.svg)](#license)
[![OpenSSF Scorecard](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/scorecard.svg)](https://scorecard.dev/viewer/?uri=github.com/wickra-lib/wickra-zk)
[![OpenSSF Best Practices](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/best-practices.svg)](https://www.bestpractices.dev)
[![Build provenance](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/provenance.svg)](https://github.com/wickra-lib/wickra-zk/attestations)
[![Docs](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/docs.svg)](https://wickra.org)
[![Zero-knowledge](https://img.shields.io/badge/proof-zero--knowledge-8b5cf6)](#what-is-proved--what-stays-private)

**Prove a backtest in zero knowledge. The deterministic Wickra engine runs inside a RISC Zero zkVM guest, and the receipt proves the report's hash and headline metrics without revealing the candles or the strategy.**

> **Part of the [Wickra ecosystem](https://github.com/wickra-lib):** the same
> data-driven core also powers
> [wickra-backtest](https://github.com/wickra-lib/wickra-backtest),
> [wickra-proof](https://github.com/wickra-lib/wickra-proof),
> [wickra-verify](https://github.com/wickra-lib/wickra-verify) and 20 more — see
> [the full list](https://github.com/wickra-lib).

**wickra-zk** runs a deterministic [Wickra](https://github.com/wickra-lib/wickra)
backtest as a guest program inside the [risc0](https://risczero.com) zkVM and
produces a succinct **zero-knowledge proof** over it. Anyone holding the proof
-- a server, an auditor, a browser -- checks it against the pinned guest and
can trust the reported performance metrics **without ever seeing the price
data or the strategy internals**. The proof is a constant-size STARK receipt;
wrapping it for an on-chain verifier is on the [roadmap](ROADMAP.md).

This works only because Wickra computes byte-deterministically: the report the
guest proves is exactly the one `wickra-backtest` produces natively and
`wickra-proof` canonicalizes and hashes.

```rust
use wickra_zk_host::{commit_dataset, prove, verify, ProveOptions, ZkSpec};

// The strategy and the candles are the private inputs; the proof is bound to
// the candles through their canonical hash, which the guest recomputes.
let spec = ZkSpec { strategy, dataset_commitment: commit_dataset(&candles)? };
let proof = prove(&spec, &candles, ProveOptions::default())?;

// Anyone verifies the receipt against the pinned guest and reads the journal
// from it -- the report hash, the commitment, sharpe, pnl, the trade count.
let journal = verify(&proof)?;
println!("report_hash: {}", journal.report_hash);
```

The same envelope -- `prove`, `commit`, `verify`, `version` as JSON -- is
what the CLI and every binding speak: Python, Node.js, C, C++, C#, Go, Java and
R prove and verify natively, and a WebAssembly build verifies in the browser.

## What is proved

- A public **`report_hash`** — the canonical hash of the full `BacktestReport`.
- A small set of **public metrics** (e.g. `sharpe`, `pnl`, `n_trades`).
- Bound to a specific **`GUEST_ID`** (the program that ran) so a verifier knows
  the honest engine produced the result.

What stays **private**: the OHLCV candles and the strategy internals — they are
guest inputs, never revealed by the receipt.

## Status

Early development (0.1.0, unreleased). See [ROADMAP.md](ROADMAP.md).

## Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) — host/guest split, zkVM choice, determinism chain
- [docs/ZK.md](docs/ZK.md) — receipt, journal, image ID; what is and is not proved
- [docs/DETERMINISM.md](docs/DETERMINISM.md) — why the journal hash equals the native hash
- [docs/PROVING.md](docs/PROVING.md) — dev-mode vs. production proving, timing, memory
- [docs/Cookbook.md](docs/Cookbook.md) — prove/verify recipes
- [THREAT_MODEL.md](THREAT_MODEL.md), [SECURITY.md](SECURITY.md)

## Quickstart

```bash
cargo install wickra-zk
wickra-zk prove  --spec examples/specs/momentum.json --data examples/data/BTCUSDT.csv --out momentum.proof.json
wickra-zk verify --proof momentum.proof.json
```

The prover is in-process and the compiled guest ships inside the crate, so
nothing else is installed. Add `--dev` (or `RISC0_DEV_MODE=1`) for a fast,
unsound receipt while developing; a real proof takes minutes. One runnable
example per language lives under [`examples/`](examples/README.md).

## Building everything from source

```bash
git clone https://github.com/wickra-lib/wickra-zk && cd wickra-zk
cargo build --release                     # host, CLI, C ABI, Python and Node crates
RISC0_DEV_MODE=1 cargo test --workspace   # dev-mode receipts: seconds, unsound
```

The compiled guest is committed, so this needs no risc0 toolchain. Changing
the guest does -- see [CONTRIBUTING.md](CONTRIBUTING.md) for the builder and
the reproducible rebuild CI holds it to.

## Project layout

```
crates/wickra-zk-host/     the host: prove, verify, the ZkSpec/PublicOutputs model
crates/wickra-zk-cli/      the `wickra-zk` command line
crates/wickra-zk-bench/    criterion benchmarks over the proving path
guest/methods/guest/       the guest program -- the code that runs inside the
                           zkVM and whose honest execution the receipt attests
guest/methods/             the compiled guest, committed: its ELF and image id
guest/builder/             compiles the guest through risc0-build and writes
                           the two files above; the only risc0 consumer
bindings/                  c, python, node, wasm (verify only), go, csharp,
                           java, r -- one JSON envelope over the host
examples/                  one runnable prove/verify per language
golden/                    frozen (spec, data) -> expected journals, one real
                           receipt, the case-to-dataset map
fuzz/                      libfuzzer targets over the JSON boundary
```

## Testing

Run the suites with the commands in
[Building everything from source](#building-everything-from-source).

- **`wickra-zk-host`** — the determinism chain, end to end: the native path's
  report hash, the dev-mode guest's journal, and the blessed fixtures must all
  agree. That equality is the product; if the guest and the native engine ever
  disagree, the proof attests to something other than what the engine computes.
- **The guest** is rebuilt for `riscv32im-risc0-zkvm-elf` in risc0's build
  container on every change and compared byte for byte with the committed ELF
  and image id, and linted for that target separately, because a guest that
  builds on the host proves nothing about the one that runs in the circuit.
- **Every binding** proves the golden cases through the envelope in dev-mode
  and holds the journal to the blessed report hash and metrics; the WebAssembly
  verifier and the host verify a committed real receipt, so the sound path is
  exercised on every push without a prover.
- **`fuzz/`** — libfuzzer targets over the spec and journal parsers, run as a
  time-boxed smoke in CI.
- **Nightly** — `prove.yml` runs the real proving path rather than dev mode.
  Dev mode produces an unsound receipt quickly, which is right for CI and wrong
  as the only thing ever exercised.

## Benchmarks

`crates/wickra-zk-bench` measures the proving path with criterion, and CodSpeed
reports instruction counts on every pull request. Absolute proving times depend
on the machine and on whether the receipt is real or dev-mode; the numbers worth
watching are relative, which is what CodSpeed reports.

```bash
cargo bench -p wickra-zk-bench
```

## Requirements

- Linux or macOS. risc0 has no Windows host, so neither the crates nor any
  binding build there; on Windows use WSL.
- Rust 1.90+. The prover is in-process and the compiled guest is committed, so
  building, proving and verifying need no risc0 toolchain; changing the guest
  does (see [CONTRIBUTING.md](CONTRIBUTING.md)).
- On macOS, Xcode's Metal toolchain: risc0 compiles its Metal kernels whenever
  the prover is built there. `xcodebuild -downloadComponent MetalToolchain`
  once; the crate reports `cannot execute tool 'metal'` until then.
- Per binding: Python 3.9+, Node.js 20+, a C toolchain and CMake, .NET 8 SDK,
  JDK 22+, Go 1.23+, R 4.1+ -- the floors the manifests declare.
- See [CONTRIBUTING.md](CONTRIBUTING.md) for the full verify workflow.

## Ecosystem

Part of the [Wickra](https://github.com/wickra-lib/wickra) family — each one a
data-driven core with a CLI and the same binding surface:

- [**wickra**](https://github.com/wickra-lib/wickra) — main library (Rust core + Python / Node.js / WASM bindings + a C ABI for C / C++ / C# / Go / Java / R)
- [**wickra-playground**](https://github.com/wickra-lib/wickra-playground) — a polyglot strategy playground: one StrategySpec live side by side in Python, Rust, JS and Go, entirely in the browser
- [**wickra-exchange**](https://github.com/wickra-lib/wickra-exchange) — unified market-data + execution across ten crypto exchanges
- [**wickra-backtest**](https://github.com/wickra-lib/wickra-backtest) — event-driven backtester over the Wickra core
- [**wickra-terminal**](https://github.com/wickra-lib/wickra-terminal) — the trading terminal: a TUI and a browser renderer over the stack
- [**wickra-screener**](https://github.com/wickra-lib/wickra-screener) — parallel multi-symbol screening over 514 streaming indicators
- [**wickra-radar**](https://github.com/wickra-lib/wickra-radar) — perp-universe alert radar: OI delta, funding flip, book imbalance, liquidation clusters, OI/price divergence
- [**wickra-copilot**](https://github.com/wickra-lib/wickra-copilot) — local market copilot grounded in real order-book, liquidation and funding microstructure
- [**wickra-shazam**](https://github.com/wickra-lib/wickra-shazam) — match an asset's current microstructure fingerprint against its entire history
- [**wickra-benchmark**](https://github.com/wickra-lib/wickra-benchmark) — reproducible, golden-verified benchmark suite — recompute any (strategy, dataset, report) in ten languages and confirm it byte-for-byte
- [**wickra-strategy-ci**](https://github.com/wickra-lib/wickra-strategy-ci) — Jest for trading strategies: golden-pin the report, catch regressions in CI, property-test against fuzzed data
- [**wickra-verify**](https://github.com/wickra-lib/wickra-verify) — confirm or refute a claimed backtest report against its strategy and data, in ten languages
- [**wickra-proof**](https://github.com/wickra-lib/wickra-proof) — Proof-of-Backtest: deterministic (spec, data) → report + blake3 hash, recomputable byte-for-byte in ten languages
- [**wickra-zk**](https://github.com/wickra-lib/wickra-zk) — this repository: prove a backtest zero-knowledge, verifiable anywhere without the data or the strategy
- [**wickra-impact**](https://github.com/wickra-lib/wickra-impact) — the backtester that knows you would have moved the market: agent-based fills on the real historical L2 order book
- [**wickra-darwin**](https://github.com/wickra-lib/wickra-darwin) — evolutionary strategy search at millions of backtests per second, mutating and crossing JSON specs across the 514-indicator space
- [**wickra-gym**](https://github.com/wickra-lib/wickra-gym) — a Gymnasium-compatible, microstructure-aware backtest environment with O(1) steps for deterministic RL rollouts
- [**wickra-feature-store**](https://github.com/wickra-lib/wickra-feature-store) — OHLCV and microstructure streams into ML-ready feature matrices over 514 O(1) streaming indicators

## Contributing

Pull requests welcome — see [CONTRIBUTING.md](CONTRIBUTING.md). By participating
you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).

## Security

Report vulnerabilities privately — see [SECURITY.md](SECURITY.md).

## License

Dual-licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE)
at your option.

The in-process prover links risc0's circuit crates, which depend on
[malachite](https://crates.io/crates/malachite) (LGPL-3.0-only) for big-integer
arithmetic. Every binary and package this repository publishes is built from
source available here under the licences above, which satisfies the LGPL's
relinking condition for statically linked libraries; risc0-zkvm itself ships
the same way. `deny.toml` names the exception.

## Disclaimer

This software is provided for research and educational purposes. It is not
financial advice. A zero-knowledge proof attests only to the honest execution of
the pinned guest program over the prover's inputs; it makes no claim about the
quality, provenance, or future performance of a trading strategy.

---

<p align="center">
  <a href="https://github.com/wickra-lib/wickra-zk">
    <img alt="GitHub stars" src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/stars.svg">
  </a>
  <a href="https://github.com/wickra-lib/wickra-zk/network/members">
    <img alt="GitHub forks" src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/forks.svg">
  </a>
  <a href="https://github.com/wickra-lib/wickra-zk/issues">
    <img alt="GitHub issues" src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/issues.svg">
  </a>
</p>

<p align="center">
  Built on <a href="https://github.com/wickra-lib/wickra">Wickra</a>. If it saved you time, the cheapest way to say thanks is to ⭐ the repo.
</p>

<p align="center">
  <img alt="wickra-zk star history" width="640"
       src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/star-history.svg">
</p>
