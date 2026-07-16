# Determinism

The core invariant of the whole repo:

> The zk journal's `report_hash` (from `verify(prove(...))`) is **byte-identical**
> to the native `wickra_proof` hash of `wickra_backtest::run(...)`.

The proof attests to the **same** deterministic report the rest of the Wickra
ecosystem computes. If that equality ever breaks, the guest has drifted from the
native engine — a bug, not a new "zk flavour" of the report.

## How it is pinned

`golden/expected/*.json` holds the blessed `report_hash` for each case, produced
by the **native** path (`wickra-backtest` + `wickra-proof`). The tests then close
the chain:

- `tests/golden.rs :: native_hash_matches_expected` — native path == blessed.
- `tests/golden.rs :: dev_mode_chain_closes` — guest journal == blessed.
- `tests/determinism.rs` — proving twice yields an identical journal.

## What can break byte-equality

The guest must reproduce the native computation exactly. Watch for:

1. **no_std drift** — the guest builds `wickra-backtest`/`wickra-proof` without
   `std`. Any `std`-only path (e.g. a `HashMap` iteration order, float
   formatting, `schemars` schema-gen) that changes bytes breaks the hash.
2. **Rounding** — metrics are rounded with `round_to(x, 1e-8)` in both the guest
   and the host; the same rule must apply on both sides.
3. **serde field order** — the canonical hash is over `canonicalize(report)`, so
   the report's serialized shape must match the native one.

## Status

The guest depends on a `no_std` build of the published `wickra-core` /
`wickra-backtest` / `wickra-proof`. Until that lands (and a risc0 toolchain is
available), the guest does not compile and the chain is verified only on its
**native** half; see [../CONTRIBUTING.md](../CONTRIBUTING.md).
