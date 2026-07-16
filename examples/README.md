# Examples

A minimal end-to-end prove/verify against fixed, deterministic data.

## Layout

| Path | Contents |
|------|----------|
| `rust/`  | An in-process example: build a `ZkSpec`, `prove`, `verify`, print the journal. |
| `specs/` | Valid `StrategySpec`s (`momentum`, `mean_reversion`, `crossover`). |
| `data/`  | Deterministic OHLCV series (`BTCUSDT.csv`, `ETHUSDT.csv`), `ts,open,high,low,close,volume`. |

## Run (dev-mode)

Dev-mode produces a fast, **unsound** receipt — perfect for a demo, never for a
real attestation:

```bash
cd rust
RISC0_DEV_MODE=1 cargo run
```

Expected shape:

```text
verified proof for guest <image-id>
{
  "report_hash": "...",
  "dataset_commitment": "...",
  "guest_id": "...",
  "sharpe": ...,
  "pnl": ...,
  "n_trades": ...
}
```

## From the CLI

Equivalently, with the `wickra-zk` binary:

```bash
wickra-zk prove --spec ../specs/momentum.json --data ../data/BTCUSDT.csv --out proof.json --dev
wickra-zk verify --proof proof.json
```

> Dev-mode (`--dev` / `RISC0_DEV_MODE=1`) is for demos only. Verifiers must
> reject dev-mode receipts in production — see [`../SECURITY.md`](../SECURITY.md).
