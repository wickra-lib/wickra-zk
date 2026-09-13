# Cookbook

Task-oriented recipes. All dev-mode commands need `RISC0_DEV_MODE=1`.

## Prove a backtest

```bash
RISC0_DEV_MODE=1 wickra-zk prove \
  --spec examples/specs/momentum.json \
  --data examples/data/BTCUSDT.csv \
  --out proof.json --dev
```

The spec may be a full `ZkSpec` or a bare `StrategySpec`; if it omits
`dataset_commitment`, the CLI computes it from the data.

## Verify a proof

```bash
wickra-zk verify --proof proof.json
# {"verified": true, "journal": { "report_hash": "...", "sharpe": ..., ... }}
```

Exit code is non-zero when `verified` is false, so it drops straight into a CI
gate.

## Get the pinned image id

```bash
wickra-zk version
# {"version": "0.1.0", "guest_id": "<hex image id>"}
```

A verifier pins this `guest_id` and rejects receipts for any other program.

## Prove from Rust

```rust
use wickra_zk_host::{commit_dataset, prove, verify, ProveOptions, ZkSpec};

let spec = ZkSpec { strategy, dataset_commitment: commit_dataset(&candles)? };
let proof = prove(&spec, &candles, ProveOptions { dev_mode: true })?;
let journal = verify(&proof)?;
```

## Prove from any binding

Every binding hands a JSON envelope to the same host. The dataset commitment
may be left out of the spec -- the host computes it -- and `commit` hashes
candles on the verifying side. In Python:

```python
import json, wickra_zk

prover = wickra_zk.Prover()
proof = json.loads(prover.command(json.dumps({"cmd": "prove", "spec": {"strategy": strategy}, "candles": candles})))
journal = json.loads(prover.command(json.dumps({"cmd": "verify", "proof": proof})))
assert journal == proof["journal"]
```

The envelope is documented once, in [ZK.md](ZK.md#the-command-envelope);
`examples/` has the same prove-and-verify in every language.

## Verify in a browser

The WebAssembly build (`bindings/wasm`) is the host without its prover: it
verifies a proof it is handed, commits to candles the page holds, and names
the guest it pins. It cannot prove -- a `prove` command is refused in-band.
See [../bindings/wasm/README.md](../bindings/wasm/README.md).

## Bless a golden case

Re-derive `golden/expected/*.json` from the native path when the engine
legitimately changes — never edit them by hand. See
[../golden/README.md](../golden/README.md).

## Run a real (sound) proof

Drop dev-mode. Expect minutes and gigabytes; this is the nightly `prove.yml`
path. See [PROVING.md](PROVING.md).
