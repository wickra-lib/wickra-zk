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

## The command envelope

Every binding -- C, Python, Node, Go, C#, Java, R -- drives the host through one
string-in/string-out call: a JSON command goes in, a JSON response comes out.
The CLI speaks the same language over files.

| Command | Payload | Response |
|---------|---------|----------|
| `prove` | `{spec: {strategy, dataset_commitment?}, candles: [...]}` | a `ZkProof`: `{receipt, journal, version}` |
| `commit` | `{candles: [...]}` | `{dataset_commitment}` |
| `verify` | `{proof: <ZkProof>}` | the `PublicOutputs`: `{report_hash, dataset_commitment, guest_id, sharpe, pnl, n_trades}` |
| `version` | -- | `{version, guest_id}` |

`strategy` is a `wickra-backtest` `StrategySpec`; `candles` is an array of
`{time, open, high, low, close, volume}`. The `dataset_commitment` in a `prove`
spec is optional: a caller that holds the candles need not carry a canonical
hasher, the host computes the commitment the same way the guest recomputes it.
A caller that states one is held to it -- a commitment that is not the hash of
the candles is refused before the zkVM runs. `commit` exists for the other side:
a verifier who holds the data can check what a proof is bound to.

`verify` returns the outputs decoded from the receipt, never the copy persisted
beside it; a proof file whose `journal` disagrees with its receipt is refused.

Errors are the host's `Error` rendered as text: `parse: ...` for a malformed
envelope, `prove: ...`, `verify: ...`, `data: ...`, or `commitment mismatch`.
The C ABI, and every binding on it (Go, C#, Java, R), reports them in-band as
`{"ok":false,"error":"..."}`; Python raises `ValueError` and Node throws.
