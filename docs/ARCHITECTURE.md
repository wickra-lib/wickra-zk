# Architecture

wickra-zk has two halves: a **host** (normal Rust, runs anywhere) and a **guest**
(a RISC-V program that runs inside the risc0 zkVM and whose honest execution a
receipt attests to).

## Crates

| Crate | Kind | Role |
|-------|------|------|
| `guest/methods/guest` | detached RISC-V bin | The program that runs in the zkVM: bind data → backtest → hash → commit journal. |
| `guest/methods` | build wrapper | `risc0_build::embed_methods()` compiles the guest to an ELF and exports `WICKRA_ZK_GUEST_ELF` / `WICKRA_ZK_GUEST_ID`. |
| `crates/wickra-zk-host` | library | `prove` / `verify` / `command_json`, the data model, the guest↔host journal. |
| `crates/wickra-zk-cli` | binary (`wickra-zk`) | reference consumer: prove/verify from files. |
| `crates/wickra-zk-bench` | benches | criterion (exec vs prod). |

The guest is **not** a workspace member — it targets `riscv32im-risc0-zkvm-elf`
and is built by the methods crate's `build.rs`, not the host's `cargo build`.

## Flow

```text
prove(spec, candles):
  host  → ExecutorEnv.write(strategy, candles, dataset_commitment)
  guest → assert hash(candles) == dataset_commitment
        → report = wickra_backtest::run(strategy, candles)
        → commit GuestJournal{ report_hash = wickra_proof(report), metrics, ... }
  host  → receipt + PublicOutputs (journal + pinned guest_id)

verify(proof):
  host  → receipt.verify(WICKRA_ZK_GUEST_ID)   # wrong program / forged journal fail here
        → decode journal, re-bind guest_id
```

## Why a zkVM (and risc0)

A zkVM lets us prove the honest execution of an *existing, audited* Rust program
— the real `wickra-backtest` engine — with no reimplementation and therefore no
"the proof used a different formula" gap. risc0 gives a mature RISC-V zkVM, a
`std`-like guest, and an image ID that cryptographically names the program. The
alternative (hand-writing a circuit) would fork the engine and reintroduce the
exact trust gap the proof is meant to close.

See [ZK.md](ZK.md) for what a proof reveals, [DETERMINISM.md](DETERMINISM.md) for
the hash-equality invariant, and [PROVING.md](PROVING.md) for dev vs prod.
