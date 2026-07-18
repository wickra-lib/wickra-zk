# What a proof proves

A wickra-zk proof is a risc0 **receipt** plus a small public **journal**. It lets
a verifier trust a backtest's headline numbers without seeing the inputs.

## Proved (public — in the journal)

- `report_hash` — the canonical `wickra-proof` hash of the full backtest report.
- `dataset_commitment` — a hash binding the proof to the exact candle series.
- `guest_id` — the image ID of the program that ran.
- `sharpe`, `pnl`, `n_trades` — selected metrics, rounded to 1e-8.

## Private (never leaves the guest)

- The candle series (price history).
- The `StrategySpec` (parameters and logic).
- The full report (trades, equity curve, all other metrics).

## The objects

| Object | What it is |
|--------|-----------|
| **Receipt** | The cryptographic proof. `receipt.verify(image_id)` succeeds only for an honest run of *that* program. |
| **Journal** | The bytes the guest committed — the public outputs above. Read from the receipt. |
| **Image ID** (`guest_id`) | A hash of the guest ELF. It names *which* program produced the receipt; a verifier pins the expected value. |

## Trust model in one line

> A valid receipt for the expected `guest_id` means: *the audited Wickra engine
> really produced these metrics and this `report_hash` from some data whose
> commitment is `dataset_commitment`* — and nothing about that data or strategy
> is revealed.

What it does **not** say: that the strategy is good, or that the candles are real
market data. See [../THREAT_MODEL.md](../THREAT_MODEL.md).
