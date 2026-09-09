# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **The guest and the host rounded differently, and the golden test's tolerance
  hid it.** `round_to(x, 1e-8)` divided by the precision and multiplied back,
  while the guest's `round8` scaled up and back down. Those are the same
  arithmetic on paper and a different `f64` in practice: for a pnl of
  `1483.61984049` the host produced `1483.6198404900001`. The guest commits its
  figures into the journal and the host recomputes its own, so the two have to
  agree bit-for-bit -- and `model.rs` already documented the guest's expression
  as the contract, so the implementation contradicted its own doc comment. The
  implementation now matches it. The golden test compares with `< 1e-8`, which
  is why this survived.

- **The guest ran on a zkVM it was not built for.** It pinned `risc0-zkvm 1.2`
  while the host executes with 3.0, so every proof aborted at runtime:
  `Invalid trap address: 0x00000000, cause: IllegalInstruction(0xc0001073, 287)`.
  The guest is on 3.0 with the workspace. This also clears the **critical**
  advisory against risc0-zkvm 1.x (arbitrary code execution in the guest via a
  memory-safety failure in `sys_read`, patched in 2.3.2).

- **`taiki-e/install-action` was pinned to a commit that does not exist**, so
  Coverage and Fuzz never got past `Set up job`:
  `Unable to resolve action ... unable to find version 6f1af7f8`. Repinned to
  the v2.87.9 commit, which is a real object.

- **Three internal path dependencies carried no version**, which `cargo-deny`
  reads as wildcards and denies. They now declare `version = "0.1.0"` beside
  the path.

- **`Clippy (host)` could not compile the guest.** `cargo clippy` exports
  `RUSTC_WORKSPACE_WRAPPER`; the child cargo that `risc0-build` spawns inherits
  it, and clippy-driver then builds the guest against a stable sysroot with no
  `std` for the zkVM target. The job sets `RISC0_SKIP_BUILD`, which is what that
  variable is for -- the guest is still compiled for real by `guest-build` and
  linted by `clippy-guest`.

- **Golden re-bless.** Only `report_hash` moves: `pnl`, `sharpe` and `n_trades`
  are byte-identical to the previous fixtures. The hash covers the whole report,
  and the engine bump added `symbol` and `timeframe` to it.

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

- `deny.toml` and `osv-scanner.toml` suppress RUSTSEC-2025-0141 with a reason.
  bincode 1.3.3 is unmaintained rather than vulnerable -- development ceased
  after a harassment incident -- and it reaches this repository only as
  risc0-zkvm's serialisation format, so there is nothing here to replace and no
  patched version to move to.

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
