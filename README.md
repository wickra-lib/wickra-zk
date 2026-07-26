<p align="center">
  <img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp" alt="Wickra" width="100%">
</p>

# wickra-zk

[![Built on Wickra](https://img.shields.io/badge/built%20on-wickra-3b82f6)](https://github.com/wickra-lib/wickra)
[![Status](https://img.shields.io/badge/status-pre--release-orange)](https://github.com/wickra-lib/wickra-zk)
[![CI](https://github.com/wickra-lib/wickra-zk/actions/workflows/ci.yml/badge.svg)](https://github.com/wickra-lib/wickra-zk/actions/workflows/ci.yml)
[![CodeQL](https://github.com/wickra-lib/wickra-zk/actions/workflows/codeql.yml/badge.svg)](https://github.com/wickra-lib/wickra-zk/actions/workflows/codeql.yml)
[![codecov](https://codecov.io/gh/wickra-lib/wickra-zk/branch/main/graph/badge.svg)](https://codecov.io/gh/wickra-lib/wickra-zk)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![OpenSSF Scorecard](https://api.securityscorecards.dev/projects/github.com/wickra-lib/wickra-zk/badge)](https://scorecard.dev/viewer/?uri=github.com/wickra-lib/wickra-zk)
[![OpenSSF Best Practices](https://img.shields.io/badge/OpenSSF-best%20practices-3b82f6)](https://www.bestpractices.dev/)
[![Build provenance](https://img.shields.io/badge/build-provenance-8957e5)](https://github.com/wickra-lib/wickra-zk/attestations)
[![Zero-knowledge](https://img.shields.io/badge/proof-zero--knowledge-8b5cf6)](#what-is-proved--what-stays-private)
[![Docs](https://img.shields.io/badge/docs-wickra.org-3b82f6)](https://wickra.org)

> Prove your backtest — zero-knowledge, on-chain-verifiable performance without
> revealing your data or strategy.

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

## Building from source

```bash
git clone https://github.com/wickra-lib/wickra-zk && cd wickra-zk
cargo risczero install
cargo build --release
```

## Requirements

- Rust 1.88+ (host), risc0 toolchain (guest)
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
