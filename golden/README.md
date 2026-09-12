# Golden harness

The golden set pins the **determinism chain** wickra-zk exists to protect: a
proof's `report_hash` must equal the hash the native `wickra-proof` computes for
the same `(strategy, data)` pair, and the journal metrics must match a native
backtest. If any link drifts, `tests/golden.rs` fails.

## Layout

| Directory | Contents |
|-----------|----------|
| `data/`     | `sym-01.csv`, `sym-02.csv`, `sym-03.csv` — fixed deterministic OHLCV universes (`ts,open,high,low,close,volume`, ≥60 bars), byte-compatible with the wickra-backtest golden data. |
| `specs/`    | `momentum.json`, `mean_reversion.json`, `crossover.json` — small, valid `StrategySpec`s. |
| `expected/` | `<case>.json` — the blessed `{report_hash, sharpe, pnl, n_trades}` for each case. |

### Case → dataset mapping

`cases.json` maps each case to its dataset; every golden test, in Rust and in
each binding, reads that file rather than carrying its own copy of the table.

| Case | Strategy | Dataset |
|------|----------|---------|
| `momentum`       | fast/slow SMA cross | `sym-01` |
| `mean_reversion` | RSI thresholds      | `sym-02` |
| `crossover`      | EMA cross           | `sym-03` |

## Data formula

Each series is a deterministic path — a smooth trend with a bounded oscillation
so trades actually open and close — generated once and committed verbatim. The
data is never regenerated at test time; the CSV bytes are the fixture.

## Bless

`expected/*.json` are produced by running the **native** path (`wickra-backtest`
+ `wickra-proof`), not by hand:

```text
report_hash = blake3_hex(canonicalize(wickra_backtest::run(&spec, &candles)))
sharpe, pnl, n_trades = round_to(metrics, 1e-8)
```

> `report_hash` is the native wickra-proof hash. The zkVM guest must reproduce
> it byte-for-byte — that equality is the whole point of the proof. **Never edit
> `expected/*.json` by hand;** re-bless from the native run if the engine
> legitimately changes.
