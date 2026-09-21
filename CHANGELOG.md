# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **The guest is built at wickra-core 1.0.5.** The host and wasm locks moved in #62; the guest's lock, bound to the
  committed artefact, follows here with the artefact risc0's container builds from it. wickra-core 1.0.5 changes no
  crate code, so the guest computes what it computed -- but a different build is a different program, and the image id
  moves from `cbdd1019…` to **`5fd57f51d640fb0da2ef44ae506e9c087d60ace76c644cfdef4776bb623e5122`**. The committed
  real receipt (`golden/proofs/momentum.json`) is re-proved for the new guest; its journal is the blessed one, byte for
  byte, only the `guest_id` it carries moves.

## [0.1.2] - 2026-09-18

### Fixed

- **The Java binding loads the library it ships.** The jar carries the native
  library under `native/<os>-<arch>/` -- the release pipeline stages every
  platform there -- but the loader only ever looked at `-Dnative.lib.dir` and
  the working directory, so a Maven Central consumer got a jar it could not
  load without pointing the JVM at a library it had to build itself. The loader
  now resolves in wickra's order: `-Dnative.lib.dir` when set, the bundled copy
  extracted to a temporary file, every `target/release` or `target/debug` up
  the tree from the working directory and the class's own location, then the
  bare name.

### Changed

- **Family pins follow the owners' releases:** wickra-backtest =0.1.4 -> =0.1.7, wickra-proof-core =0.1.2 -> =0.1.3, in the host
  and in the guest. A different guest source is a different program: the docker rebuild produced a new artefact, and the image id
  moves from `542491b0…` to **`cbdd10199ba8c50e3923c2d3be331e63f464293c5db5335335a35c9e59ef29c1`** — a verifier that pins the
  old id rejects receipts from this release, which is the point of pinning. The committed real receipt (`golden/proofs/momentum.json`)
  is re-proved for the new guest; its journal is the blessed one, byte for byte. No code of this repository changes.
- **Every README follows wickra's shape.** A cross-repo scan compared the
  heading skeleton of each README against wickra's and this repository's
  differed throughout. The root README opens as wickra's does (banner, badges,
  the one-liner, the live-demo and ecosystem lines, no separate H1), the
  License section carries wickra's wording and its `### Contribution` clause,
  and the shared sections run in wickra's order. Each binding README is
  `Install`, `Quick start`, `Benchmark`, `Documentation`, `Security`,
  `Disclaimer`, `License` with the product's own surface and protocol notes
  as subsections; the registry pages that render them now say how to report a
  vulnerability and under which licence the package ships.
  `examples/README.md` lists every language the way wickra's does, with the
  commands the CI examples job runs; the per-language example READMEs,
  `fuzz/README.md` and the `## Editing the docs` section of
  `docs/README.md` exist as they do in wickra.

### Changed

- **The wasm binding's lock says what its manifests say.** `bindings/wasm/Cargo.lock`
  was committed with the crates at 0.1.0 and `wickra-proof-core` from a git rev
  the workspace had long replaced with crates.io -- a detached workspace that
  nothing in the bump refreshes. It is re-resolved (0.1.1, registry sources);
  Dependabot now covers that directory so its own dependencies move too.
  `examples/rust` named `wickra-zk-host` at 0.1.0 for the same reason and says
  0.1.1.
- **The repository spells shared things the way the family does.** A cross-repo
  scan lined the 24 wickra-lib repositories up and this one also differed in:
  `napi = "3"` / `napi-derive = "3"` where the family writes `3.9` / `3.5`,
  `engines.node >= 20` in the Node packages where every sibling requires 22,
  the C example's `CMAKE_CXX_STANDARD` 14 where the family builds with 17,
  `dotnet-version: "8.0.x"` in the example job, the fuzz job on a rolling
  nightly rather than the family's pinned `nightly-2026-07-01`, and
  `examples/node` absent from Dependabot. The Node lock gains the four platform
  packages, published since v0.1.1.

  `wickra-backtest` stays at `=0.1.4` for now: `wickra-proof-core` 0.1.2 pins
  `wickra-backtest-core` to that exact version, so the move to 0.1.6 comes with
  wickra-proof 0.1.3, together with the guest rebuild it implies.

### Changed

- **The link check covers NuGet and pkg.go.dev.** Both time-boxed `lychee`
  excludes written before the first release are gone: `Wickra.Zk` and its
  four runtime packages answer on nuget.org, and pkg.go.dev indexes
  `wickra-zk-go` since v0.1.1 (verified: 200 each), so the README badges
  are checked like every other published link.

- **`BENCHMARKS.md` carries measurements.** The first nightly bench that
  finished (2026-09-15) replaces the placeholder: execution 237 ms, a real
  proof 17 min 37 s, verification 19 ms on the `momentum` case, on a
  GitHub-hosted runner, with what each number depends on and how to
  reproduce it.

### Fixed

- **The nightly benchmark finishes.** `bench.yml` measured real proving with
  Criterion's minimum of ten samples at roughly fourteen minutes a proof under
  a two-hour budget, so every nightly run was cancelled and `BENCHMARKS.md`
  never received a number. The job has the runner's six-hour budget, and the
  suite gains `verify/<case>`, the third measurement the document promised:
  one real receipt produced outside the timed loop, checked against the
  pinned guest id.

- **The pinned `uv` bootstrap could not verify its download.**
  `scripts/update-lockfiles.sh` named uv 0.12.14 but kept the release
  checksums of 0.12.13, so `WICKRA_BOOTSTRAP_UV=1` fetched the
  archive and then refused it. The pin and all four checksums now name
  0.12.15, taken from the release's `.sha256` files.

## [0.1.1] - 2026-09-15

### Fixed

- **The NuGet package publishes.** NuGet.org caps a package at 250 MB and the
  prover library is about 90 MB per platform, so `Wickra.Zk` with all four
  runtimes inside was refused at upload (413). The native libraries now ship as
  `Wickra.Zk.runtime.<rid>`, one package per runtime identifier, and
  `Wickra.Zk` depends on all four: one `dotnet add package` still brings every
  platform, and `dotnet run` without a `RuntimeIdentifier` still finds the
  library through `deps.json`. The release pipeline packs the runtime packages
  first and pushes them ahead of the main package.

### Changed

- **`float_roundtrip` is stated, not inherited.** The workspace and the guest
  now declare serde_json's correctly rounded float parser themselves.
  `wickra-proof-core` already requires it and cargo unifies the feature across
  the graph, so nothing in the built binaries or the guest ELF changes; the
  decision just no longer lives in the engine's manifest.

## [0.1.0] - 2026-09-14

### Changed

- **The Python 3.9 CI row installs no pytest.** pytest 9.x requires 3.10, so
  the 3.9 row could only pin 8.4.2, which is below the fix for
  GHSA-6w46-j5rx-g56g and has no backport. The dev requirements are locked
  twice now (`ci-dev-py3.txt`, `ci-dev-py39.txt`, both hash-pinned), the 3.9
  lock carries maturin alone, and the row runs the suite through
  `run_without_pytest.py` -- the same modules, rewritten as plain functions
  with plain asserts, which 3.10 and up still run under pytest.

- **The compiled guest is committed, and the prover runs in-process.** The
  methods crate built the guest in its `build.rs`, so every crate above it --
  and every wheel, npm package and C ABI archive built from them -- needed the
  risc0 toolchain, docs.rs could never build the host, and the crate's
  `publish = false` made the host unpublishable in turn. `guest/builder`, the
  one crate that needs risc0, now writes `guest/methods/elf/wickra-zk-guest.bin`
  and the image id in `guest/methods/src/guest_id.rs`; the `guest-build` job
  rebuilds the guest with `RISC0_USE_DOCKER=1` and fails on a diff. risc0's
  default prover runs `r0vm` as a subprocess even for dev-mode receipts; with
  risc0-zkvm's `prove` feature the prover is in-process, so a `pip install` or
  a `cargo install` is the whole prover. The in-process prover links
  `malachite` (LGPL-3.0-only); the README's License section says how that is
  met.

- **The command envelope is usable from a binding.** `dataset_commitment` was
  required in every `prove` spec and only the host can compute it, so no
  binding could prove. It is optional now -- the host computes it, a stated one
  is held to -- and a `commit` command hashes candles for the verifying side;
  `version` names the guest id. `verify` compared only the persisted
  `guest_id` with the receipt, so a proof file whose journal copy lied about
  `report_hash` verified; the whole persisted journal is checked now.

- **Linux and macOS only.** risc0 has no Windows host and the host crate does
  not link on MSVC; no CI matrix, release matrix or package targets Windows.
  The R package is `OS_type: unix`. MSRV is Rust 1.90.

- **The C# package is `Wickra.Zk`.** It was `Wickra.Proof` -- wickra-proof's
  package id, namespace and assembly name -- by copy.

### Added

- **Every binding tests both operating modes of `prove`.** The caller can
  leave the dataset commitment to the host or state it up front -- the value
  `commit` returns. The journal must not depend on which, and both proofs must
  verify to it. Python, Node, C#, Go, Java, R and the C ABI test now re-prove
  every golden case with the stated commitment and compare; the WASM verifier
  checks the other pair of modes it has -- the committed proof bytes and the
  proof as the host's JSON library re-emits it decode to one journal.
- **A browser demo joins the examples.** `examples/wasm/verify.html` fetches a
  real golden receipt and decodes its journal through the WebAssembly
  verifier; the Examples job parse-checks its module.

- Golden tests in every binding: each of Python, Node, Go, C#, Java, R and C
  proves the three golden cases through the envelope in dev-mode and holds the
  journal to the blessed `report_hash` and metrics, verifies the proof, refuses
  a proof whose journal lies, and refuses a false commitment. `golden/cases.json`
  is the one case-to-dataset mapping every test reads. `examples/c` is the C
  ABI's first test.
- One runnable example per language under `examples/`, all proving the same
  inputs to the same `report_hash`, which the Examples job holds them to.
- The release front of the family: guard, gate, crates.io (methods, host,
  CLI), PyPI wheels and sdist, npm with per-platform packages, NuGet, Maven
  Central, the Go mirror (`wickra-lib/wickra-zk-go`), C ABI archives, CLI
  binaries, the guest ELF and image id, SBOMs and build provenance.
- CI builds risc0's C++ kernels with clang. One of them,
  `risc0-circuit-keccak-sys`, is a single generated file that g++ 13 spends
  25 minutes optimising on a runner core and clang 18 compiles in seconds;
  every runner job that built the prover cold took over half an hour because
  of it. The Examples job builds the prover once for every language (the napi
  CLI's explicit `--target` used to send it into a second cold build), the
  container wheel smoke covers the four Linux wheels the release ships, and
  the macOS wheel builds get the Metal toolchain risc0-sys needs.

### Fixed

- The strategy crosses the host/guest boundary as JSON text. risc0's serde is
  a word stream, and `StrategySpec` relies on JSON semantics
  (`skip_serializing_if` on an `Option`, an `untagged` enum), so written as a
  value it never read back: every dev-mode prove died in the guest with
  `DeserializeBadOption`.
- The node binding did not compile: cargo passes the workspace
  `unsafe_code = "forbid"` on the command line and napi-derive's `allow`
  cannot overrule it (E0453). The crate declares its own lints now.
- The R `configure` read `WKPROOF_INC` / `WKPROOF_LIB`; CI sets `WKZK_*`.
- The Node tests were ESM syntax over a CommonJS `require` and could not run.
- cargo-deny: `ruint` 1.20.0 fixes RUSTSEC-2026-0220; `rsa`, `derivative` and
  `paste` have no fixed release and are ignored with their chain and reason.
- Code scanning: every action pinned at patch level, the risc0 setup action
  without template expansion or `GITHUB_PATH`, Dependabot cooldowns on every
  ecosystem, the Python CI tools installed from the hash-locked requirements.

- **Follows wickra-proof's core crate rename.** `proof-core` became
  `wickra-proof-core` upstream because the old name was taken on crates.io by
  an unrelated project before wickra-proof had released -- so its first publish
  would have failed with a permission error after the tag. The guest and the
  host now call `wickra_proof_core::`, and the pin moves to the commit that
  carries the rename.

- **`serde_with` 3.17.0 -> 3.22.0**, clearing GHSA-7gcf-g7xr-8hxj: `KeyValueMap`
  serialisation panicked on empty sequence or map entries. Fixed from 3.21.0.

- **The remaining advisory is suppressed with its reason, because it cannot be
  fixed here.** `tracing-subscriber` 0.2.25 carries CVE-2025-58160 -- ANSI
  escape sequences can poison log output when untrusted input reaches a log
  line -- and the fix is 0.3.20. That version is unreachable: `ark-relations`
  declares the 0.2 line, and the chain to us runs ark-relations <-
  ark-crypto-primitives <- ark-groth16 <- risc0-groth16 <- risc0-zkvm. A pin
  that blocks an update raises no PR and reports nothing; recording why is the
  only honest thing to do with it. Nothing here logs untrusted input through
  that subscriber either -- it arrives with risc0's groth16 path, which this
  crate does not drive.

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
  `wickra_proof_core::hash_candles` and `wickra_proof_core::hash_report`, two functions that
  did not exist in `wickra-proof` and never had; the host's own doc comments
  called them *"provisional"* and the guest's module doc called itself
  *"blind-authored"*. CI has 30 runs and no successful one. wickra-proof now
  exports those entry points, and both sides call them.

  The guest also read `report.sharpe`, `report.pnl` and `report.n_trades`
  directly off `BacktestReport`. Those figures live on `report.metrics`, and the
  trade count is `num_trades: usize`, so the journal now reads them from there
  and converts with `u64::try_from`.

- **The git dependencies floated.** `wickra-backtest` and `wickra-proof-core` were
  taken from a branch with no `rev`, so they tracked whatever upstream had last
  pushed. That is how the guest came to call functions that no longer existed:
  an unpinned git dependency never goes red, it only gets older, and then one
  day the API underneath has moved and the failure looks like your own. Both are
  pinned to exact published releases now, `wickra-backtest =0.1.4` and
  `wickra-proof-core =0.1.2`, in the host and the guest alike (the two must
  agree: proof-core 0.1.2 requires `wickra-backtest-core =0.1.4`, and two
  copies of the engine in one graph do not share a `BacktestReport`), and
  `deny.toml` no longer allows git sources at all.

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

[Unreleased]: https://github.com/wickra-lib/wickra-zk/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/wickra-lib/wickra-zk/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/wickra-lib/wickra-zk/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/wickra-lib/wickra-zk/releases/tag/v0.1.0
