# Contributing to wickra-zk

Thanks for your interest in improving wickra-zk — a zero-knowledge prover and
verifier for deterministic Wickra backtests. This project runs the backtest as a
guest program inside the [risc0](https://risczero.com) zkVM and produces a proof
that a given `report_hash` (and its public metrics) is the honest result of
running a strategy over some private data.

## Layout

- **`guest/methods/guest/`** — the guest program: the actual code that
  runs inside the zkVM. It reads a spec and candles, runs the deterministic
  `wickra-backtest` engine, canonicalizes with `wickra-proof`, and commits the
  public outputs to the journal.
- **`guest/methods/`** — the host-side methods crate: the compiled guest,
  committed. `elf/wickra-zk-guest.bin` is the ELF the prover runs and
  `src/guest_id.rs` the image id the verifier pins (`WICKRA_ZK_GUEST_ELF` /
  `WICKRA_ZK_GUEST_ID`). It builds with plain Rust.
- **`guest/builder/`** — the one crate that needs the risc0 toolchain. It
  compiles the guest through `risc0-build` and writes the ELF and the id into
  `guest/methods`. Run it after every change to the guest source:

  ```bash
  RISC0_USE_DOCKER=1 cargo run --manifest-path guest/builder/Cargo.toml --release
  ```

  `RISC0_USE_DOCKER=1` is the reproducible build risc0 designs the image id
  around; the `guest-build` CI job rebuilds the guest the same way and fails
  when the committed artefact differs from what the source produces.
- **`crates/wickra-zk-host/`** — the prover/verifier library (`prove`, `verify`,
  `guest_id`, `command_json`).
- **`crates/wickra-zk-cli/`** — the `wickra-zk` binary (`prove` / `verify`).
- **`crates/wickra-zk-bench/`** — criterion benchmarks (exec vs. prove).

The host workspace and the guest are **separate builds**: the guest targets
`riscv32im-risc0-zkvm-elf` with its own toolchain, and neither it nor the
builder is a workspace member.

## Prerequisites

Linux or macOS: risc0 has no Windows host, and the host crate does not link on
MSVC (on Windows, use WSL). Building and testing the host, the CLI and the
bindings needs only Rust: the guest is committed and the prover is in-process
(risc0-zkvm's `prove` feature), dev-mode and real alike. Changing the guest
needs the risc0 toolchain (the guest compiler):

```bash
curl -L https://risczero.com/install | bash
rzup install cargo-risczero 3.0.6   # the risc0-zkvm crate version
rzup install rust
rzup install r0vm 3.0.6
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
# after a change to guest/methods/guest:
RISC0_USE_DOCKER=1 cargo run --manifest-path guest/builder/Cargo.toml --release
git diff --exit-code -- guest/methods/elf guest/methods/src/guest_id.rs
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
