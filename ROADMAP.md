# Roadmap

wickra-zk is pre-1.0. The current milestone is a working host/guest prover and
verifier with a golden-pinned determinism chain. Beyond that:

## Toward 1.0

- **On-chain verification** — wrap the receipt in a Groth16 SNARK and ship a
  reference on-chain verifier so a smart contract can check a backtest proof
  directly.
- **Dataset commitment** — bind the private candles to a public commitment
  (e.g. a Merkle root) in the journal, so a verifier can be sure two proofs used
  the *same* data, or that the data matches a trusted feed.
- **Strategy commitment** — commit to a hash of the strategy spec, letting a
  prover reveal *which* strategy was proven without revealing its parameters.

## Later

- **Recursion / aggregation** — fold many per-period proofs into one, so a long
  backtest or a portfolio of strategies produces a single succinct receipt.
- **Continuous proving** — prove rolling windows as new bars arrive, for live
  trustless track records.

Nothing here changes the core invariant: whatever is proven is always the same
deterministic report `wickra-backtest` computes and `wickra-proof` hashes.
