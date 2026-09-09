# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **Neither the guest nor the host had ever compiled.** Both called
  `proof_core::hash_candles` and `proof_core::hash_report`, two functions that
  did not exist in `wickra-proof` and never had; the host's own doc comments
  called them *"provisional"* and the guest's module doc called itself
  *"blind-authored"*. CI has 30 runs and no successful one. wickra-proof now
  exports those entry points, and both sides call them.

  The guest also read `report.sharpe`, `report.pnl` and `report.n_trades`
  directly off `BacktestReport`. Those figures live on `report.metrics`, and the
  trade count is `num_trades: usize`, so the journal now reads them from there
  and converts with `u64::try_from`.

- **The git dependencies floated.** `wickra-backtest` and `proof-core` were
  taken from a branch with no `rev`, so they tracked whatever upstream had last
  pushed. That is how the guest came to call functions that no longer existed:
  an unpinned git dependency never goes red, it only gets older, and then one
  day the API underneath has moved and the failure looks like your own. Both are
  pinned to an exact rev.

- **A dependency pointed at a repository that does not exist.** The workspace
  declared `wickra-data = { git = ".../wickra-lib/wickra-data" }`, which 404s --
  `wickra-data` is a crate inside the `wickra` repository. No member referenced
  it, so it never resolved and never appeared in `Cargo.lock`; it would have
  failed the build the day someone wrote `wickra-data.workspace = true`.
  Removed.

- **`Clippy (guest, riscv)` could not pass.** It ran `cargo clippy` in
  `guest/methods/guest`, where the repository-root `rust-toolchain.toml` pins
  stable 1.88 -- a toolchain with no `std` for `riscv32im-risc0-zkvm-elf` -- so
  the job died with `error[E0463]: can't find crate for std` on a guest that
  builds. `guest-build` never hit it because `risc0-build` invokes the risc0
  toolchain itself. The job now selects it explicitly with `+risc0`.

### Changed

- **The documentation no longer claims the guest cannot be built.**
  `ARCHITECTURE.md`, `CONTRIBUTING.md`, `docs/DETERMINISM.md`, the guest's
  module doc and its manifest all stated that the guest waits on a `no_std`
  conversion of `wickra-core` -> `wickra-backtest` -> `wickra-proof`. It does
  not: risc0's guest std shim carries what those crates use, and the CI log
  shows every one of them -- and `rayon` besides -- compiling for
  `riscv32im-risc0-zkvm-elf` before the guest's own five errors. The claim cost
  a day to disprove and would have cost the next reader the same.

- `commit_dataset` returns `Result<String>` rather than `String`, since
  canonicalisation can fail.

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
