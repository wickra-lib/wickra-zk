# Contributing to wickra-zk

Thanks for your interest in improving wickra-zk — a zero-knowledge prover and
verifier for deterministic Wickra backtests. This project runs the backtest as a
guest program inside the [risc0](https://risczero.com) zkVM and produces a proof
that a given `report_hash` (and its public metrics) is the honest result of
running a strategy over some private data.

## Layout

- **`guest/methods/guest/`** — the no_std guest program: the actual code that
  runs inside the zkVM. It reads a spec and candles, runs the deterministic
  `wickra-backtest` engine, canonicalizes with `wickra-proof`, and commits the
  public outputs to the journal.
- **`guest/methods/`** — the host-side methods crate; its `build.rs` compiles
  the guest to a RISC-V ELF and exports `WICKRA_ZK_GUEST_ELF` / `_ID`.
- **`crates/wickra-zk-host/`** — the prover/verifier library (`prove`, `verify`,
  `guest_id`, `command_json`).
- **`crates/wickra-zk-cli/`** — the `wickra-zk` binary (`prove` / `verify`).
- **`crates/wickra-zk-bench/`** — criterion benchmarks (exec vs. prove).

The host workspace and the guest are **separate builds**: the guest targets
`riscv32im-risc0-zkvm-elf` with its own toolchain and is not a workspace member.

## Prerequisites

Install the risc0 toolchain (guest compiler + `r0vm` executor):

```bash
cargo install cargo-risczero
cargo risczero install
```

Most host tests run in **dev-mode** (`RISC0_DEV_MODE=1`), which produces a fake
receipt fast; it verifies the *logic* but is **not** cryptographically sound. Real
proving (no dev-mode) is exercised by the nightly `prove.yml` workflow.

## Verify before you push

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
( cd guest/methods/guest && cargo clippy --target riscv32im-risc0-zkvm-elf -- -D warnings )
RISC0_DEV_MODE=1 cargo test --workspace --all-features
cargo deny check
```

## Determinism is the whole point

The guest's report **must** be byte-identical to what `wickra-backtest` produces
natively and what `wickra-proof` canonicalizes and hashes. A journal `report_hash`
that differs from the native hash is a bug. Therefore: no `HashMap` in the report
path (`BTreeMap` only), no time/RNG/threads in the guest, and exactly the same
float rounding as `wickra-proof`.

## Ground rules

- Sign your commits (`git commit -S`) and sign off under the [DCO](DCO)
  (`Signed-off-by:` — `git commit -s`).
- One logical change per commit; open a pull request against `main` and wait for
  CI to pass before it is merged.
- Add a `## [Unreleased]` entry to [CHANGELOG.md](CHANGELOG.md).
- The project is dual-licensed `MIT OR Apache-2.0`; contributions are accepted
  under the same terms.

By contributing you agree to abide by the [Code of Conduct](CODE_OF_CONDUCT.md).
