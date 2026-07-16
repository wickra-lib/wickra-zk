# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Project scaffolding: the host workspace manifest, dual `MIT OR Apache-2.0`
  license, supply-chain and link config (`deny.toml`, `lychee.toml`,
  `osv-scanner.toml`, `repo-metadata.toml`), and the governance and community
  docs.
- Guest program (`guest/methods/guest`) and the `risc0-build` methods wrapper:
  bind the dataset commitment, run the deterministic `wickra-backtest` engine,
  hash the report with `wickra-proof`, and commit the public journal.
- `wickra-zk-host`: `prove` / `verify` / `command_json`, the `ZkSpec` /
  `PublicOutputs` / `ZkProof` model, `commit_dataset` and `native_report_hash`.
- `wickra-zk` CLI: `prove` / `verify` / `version`.
- Golden harness (`golden/`): deterministic data, three strategies, and the
  natively blessed `{report_hash, sharpe, pnl, n_trades}` the guest must
  reproduce.
- Integration tests (roundtrip, golden hash-chain, determinism, nightly
  `prod_roundtrip`), fuzz targets, and a criterion bench crate.
- Examples (`examples/`) and CI/CD workflows (CI, CodeQL, Scorecard, zizmor,
  links, sync-metadata, nightly prove/bench, tag-gated release).
- Documentation set: `docs/{ARCHITECTURE,ZK,DETERMINISM,PROVING,Cookbook}.md`.

[Unreleased]: https://github.com/wickra-lib/wickra-zk/commits/main
