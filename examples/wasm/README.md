# wickra-zk WASM examples

Browser demos for the `wickra-zk-wasm` binding -- the verifying side. Proving
runs a zkVM and belongs to a native process; a browser verifies what it is
handed: a RISC Zero receipt and its public journal, checked against the guest
this build pins, without a prover, without the candles and without the
strategy.

## Build

The module ships as a `wasm-pack` `--target web` bundle. Build it once from the
repository root:

```bash
wasm-pack build bindings/wasm --target web --release
```

That writes `bindings/wasm/pkg/` with the `.wasm` binary, the JS loader and the
TypeScript types. The demo imports the loader via
`../../bindings/wasm/pkg/wickra_zk_wasm.js`.

## Serve

ES-module imports and `fetch` need a real HTTP origin, not `file://`. Any
static server from the repository root works:

```bash
python -m http.server 8000
```

Then open `http://localhost:8000/examples/wasm/verify.html`.

## Demos

| File | What it does |
| --- | --- |
| `verify.html` | Fetches `golden/proofs/momentum.json`, a receipt the real prover made for the momentum golden case, verifies it and shows the journal decoded from the receipt -- `report_hash`, `dataset_commitment`, `guest_id`, `sharpe`, `pnl`, `n_trades`. The page counterpart of the `verify` half of `examples/node/prove.js`. |

## See also

- [examples/README.md](../README.md) -- the same proof produced and verified in every other language.
- [bindings/wasm/README.md](../../bindings/wasm/README.md) -- the module's API and what it does not carry.
