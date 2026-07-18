# Proving: dev-mode vs production

## Dev-mode (fast, NOT sound)

```bash
RISC0_DEV_MODE=1 cargo run -p wickra-zk -- prove --spec s.json --data d.csv --out p.json --dev
```

Dev-mode runs the guest and produces the real journal, but the receipt is a
**fake** — it is not a cryptographic proof. Use it for development, tests and CI
(the `golden` / `cli-roundtrip` jobs). A verifier **must reject** dev-mode
receipts in production; see [../SECURITY.md](../SECURITY.md).

## Production (sound, slow)

Drop `RISC0_DEV_MODE` and `--dev`. The prover generates a real receipt:

```bash
wickra-zk prove --spec s.json --data d.csv --out p.json
```

Real proving is CPU- and memory-heavy (minutes, gigabytes), so it runs nightly
(`prove.yml`, `tests/prod_roundtrip.rs`) rather than on every push. The public
outputs are identical to dev-mode — a real proof does not change *what* is
proved, only *whether it is trustless*.

## Timing and memory

Numbers land in [../BENCHMARKS.md](../BENCHMARKS.md) from the nightly `bench.yml`
(`exec/*` = guest execution in dev-mode; `prove/*` = real proving). Execution and
verification are cheap; proving dominates.

## On-chain outlook

A receipt can be wrapped in a Groth16 SNARK small enough for an on-chain
verifier, so a smart contract can check a backtest proof directly. That, plus an
explicit dataset commitment, is on the [../ROADMAP.md](../ROADMAP.md).
