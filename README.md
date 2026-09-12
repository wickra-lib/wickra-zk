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

> Prove your backtest — zero-knowledge, on-chain-verifiable performance without
> revealing your data or strategy.

> **Part of the [Wickra ecosystem](https://github.com/wickra-lib):** the same
> data-driven core and ten-language binding surface also power
> [wickra-backtest](https://github.com/wickra-lib/wickra-backtest),
> [wickra-proof](https://github.com/wickra-lib/wickra-proof),
> [wickra-verify](https://github.com/wickra-lib/wickra-verify) and 20 more — see
> [the full list](https://github.com/wickra-lib).

**wickra-zk** runs a deterministic [Wickra](https://github.com/wickra-lib/wickra)
backtest as a guest program inside the [risc0](https://risczero.com) zkVM and
produces a succinct **zero-knowledge proof** over it. A verifier — on a server or
on-chain — can check the proof and trust the reported performance metrics
**without ever seeing the price data or the strategy internals**.

This works only because Wickra computes byte-deterministically: the report the
guest proves is exactly the one `wickra-backtest` produces natively and
`wickra-proof` canonicalizes and hashes.

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
cargo install cargo-risczero && cargo risczero install
wickra-zk prove  --spec examples/specs/momentum.json --data examples/data/BTCUSDT.csv --out momentum.proof.json
wickra-zk verify --proof momentum.proof.json
```

## Building everything from source
```bash
git clone https://github.com/wickra-lib/wickra-zk && cd wickra-zk
cargo risczero install
cargo build --release
```

## Project layout

```
crates/wickra-zk-host/     the host: prove, verify, the ZkSpec/PublicOutputs model
crates/wickra-zk-cli/      the `wickra-zk` command line
crates/wickra-zk-bench/    criterion benchmarks over the proving path
guest/methods/             host-side wrapper; build.rs compiles the guest to a
                           RISC-V ELF and exports its image id
guest/methods/guest/       the guest program -- the code that runs inside the
                           zkVM and whose honest execution the receipt attests
golden/                    frozen (spec, data) -> expected triples
fuzz/                      libfuzzer targets over the JSON boundary
```

## Testing

Run the suites with the commands in
[Building everything from source](#building-everything-from-source).

- **`wickra-zk-host`** — the determinism chain, end to end: the native path's
  report hash, the dev-mode guest's journal, and the blessed fixtures must all
  agree. That equality is the product; if the guest and the native engine ever
  disagree, the proof attests to something other than what the engine computes.
- **The guest** is compiled for `riscv32im-risc0-zkvm-elf` on every change and
  linted for that target separately, because a guest that builds on the host
  proves nothing about the one that runs in the circuit.
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
- Rust 1.90+ (host), risc0 toolchain (guest)
- See [CONTRIBUTING.md](CONTRIBUTING.md) for the full verify workflow

## Contributing

Pull requests welcome — see [CONTRIBUTING.md](CONTRIBUTING.md). By participating
you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).

## Security

Report vulnerabilities privately — see [SECURITY.md](SECURITY.md).

## License

Dual-licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE)
at your option.

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
