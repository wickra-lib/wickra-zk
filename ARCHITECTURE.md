# Architecture

wickra-zk proves, in zero knowledge, that a public `report_hash` and a set of
public metrics are the honest result of running a deterministic Wickra backtest
over private data. It is a **host/guest** program on the
[risc0](https://risczero.com) zkVM.

## Why risc0

The engine we need to prove — `wickra-backtest` — is ordinary deterministic Rust.
A general-purpose RISC-V zkVM lets us run that exact code as the guest instead of
re-expressing the backtest as an arithmetic circuit by hand. risc0 gives us:

- a **Rust guest** running the real engine, so the proven program *is* the
  audited one and not a hand-written reimplementation that could diverge;
- a **`GUEST_ID` (image ID)** that cryptographically names the exact program, so
  a verifier knows the honest engine produced the result;
- a **journal** — the public commitment — plus a **receipt** that can be verified
  cheaply and, with a Groth16 wrapper, on-chain.

The alternative circuit-DSL zkVMs (e.g. SP1) have a different journal/receipt/
image-ID model; switching would change the on-disk proof format, so the choice is
pinned (see the version matrix in the handoff and `Cargo.toml`).

## Host / guest split

```
guest/methods/guest/   the guest program (runs inside the zkVM)
guest/methods/         host-side crate; build.rs compiles the guest to a RISC-V
                       ELF and exports WICKRA_ZK_GUEST_ELF + WICKRA_ZK_GUEST_ID
crates/wickra-zk-host  prove() / verify() / guest_id() / command_json()
crates/wickra-zk-cli   the `wickra-zk` binary (prove | verify | version)
crates/wickra-zk-bench criterion benchmarks (exec vs. prove)
```

The guest is a **detached build** (its own toolchain and `riscv32im-risc0-zkvm-elf`
target); it is deliberately **not** a workspace member. `risc0-build` runs it at
host build time via `guest/methods/build.rs`.

## The proving flow

1. The host loads a strategy spec (`ZkSpec`) and candle data (private inputs).
2. It writes them into the guest's `ExecutorEnv` and runs the prover against
   `WICKRA_ZK_GUEST_ELF`. The strategy crosses as JSON text, the candles as
   a word stream: risc0's serde carries no field names or tags, and
   `StrategySpec` relies on JSON semantics (`skip_serializing_if` on an
   `Option`, an `untagged` enum), so written as a value it does not read
   back.
3. The guest reads the inputs, runs `wickra-backtest`, canonicalizes the report
   with `wickra-proof`, and `commit`s the public outputs (the `report_hash` and
   a few metrics) to the journal.
4. The host returns a `ZkProof` (receipt + public outputs).
5. A verifier calls `receipt.verify(GUEST_ID)`; success means the journal is the
   honest output of the pinned program.

## The determinism chain

The value of the proof rests on one identity:

```
guest journal.report_hash  ==  wickra-proof hash of the native BacktestReport
```

The same engine (`wickra-backtest`) and the same canonicalization/hash
(`wickra-proof`) run in the guest and natively, so the hashes must match
byte-for-byte. This requires the guest report path to be free of nondeterminism:
`BTreeMap` (never `HashMap`), no clock/RNG/threads, and identical float rounding.
See [docs/DETERMINISM.md](docs/DETERMINISM.md). The `golden.rs` test pins this
equality; a mismatch is a bug, not a tolerance.
