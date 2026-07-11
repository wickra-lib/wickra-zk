# Threat model

wickra-zk exists to make a backtest result **trustless**: a verifier who never
sees the data or the strategy can still be sure the reported performance is
honest. This document states what that guarantee covers and what it does not.

## Assets

- **Private OHLCV data** — the price history a prover backtests over. Must never
  be revealed by a proof.
- **Strategy internals** — the parameters and logic inside the spec. Also a guest
  input, not exposed by the receipt.
- **Report integrity** — the public `report_hash` and metrics must correspond to
  the honest execution of the audited engine.

## Actors

- **Honest prover** — runs the real engine over real data and publishes a receipt.
- **Lying prover** — wants to publish attractive metrics that do not follow from
  any honest run, or that come from a different (doctored) program.
- **Verifier** — a server, an auditor, or an on-chain contract checking a receipt.

## Threats and mitigations

| Threat | Mitigation |
|--------|------------|
| Prover fakes the metrics (claims a Sharpe the engine never produced) | The receipt proves the journal is the honest output of the pinned program; a fabricated journal has no valid receipt. |
| Prover runs a *different* program (a doctored engine) | The receipt is bound to a `GUEST_ID`; the verifier checks it against the **expected** image ID. A receipt for another program fails that check. |
| Prover swaps in different data after the fact | The journal commits to a `report_hash` over the actual inputs used; a later data swap changes the hash and cannot reuse the receipt. A future `dataset_commitment` (see [ROADMAP.md](ROADMAP.md)) binds the inputs explicitly. |
| Dev-mode receipt trusted as real | A `RISC0_DEV_MODE=1` receipt is a fast fake and is **not** sound. Verifiers must reject dev-mode receipts in production; see [SECURITY.md](SECURITY.md). |
| Report format diverges from the native/`wickra-proof` hash | The `golden.rs` test pins `journal.report_hash == native hash`; CI fails on any divergence, so a proof always attests the same report the rest of the ecosystem computes. |

## Out of scope

- **Strategy quality.** A proof says the engine ran honestly; it makes no claim
  that a strategy is good, robust, or profitable in the future.
- **Data authenticity.** wickra-zk proves computation over the inputs it was
  given; it does not attest that those candles are real market data. Binding to a
  trusted data source is a separate (future) commitment.
- **Side channels of the prover host.** Protecting the prover's own machine and
  the secrecy of its inputs at rest is the operator's responsibility.
