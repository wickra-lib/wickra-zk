# Wickra ZK — WASM

The verifying side of `wickra-zk`, compiled to WebAssembly with wasm-bindgen: a
browser receives a proof -- a RISC Zero receipt and its public journal -- and
checks it against the guest this build pins, without a prover, without the
candles and without the strategy.

Create a `Verifier`, drive it with a command JSON and read back the response
JSON: the same envelope every other binding speaks, minus `prove`. Proving runs
a zkVM and belongs to a native process; a browser verifies what it is handed.

## Build

```bash
wasm-pack build --target web            # pkg/  -- what ships
wasm-pack build --target nodejs --out-dir pkg-node   # for the tests
cargo test                              # the crate's own unit tests, natively
```

This crate is its own workspace: the host it wraps is built without its
`prove` feature, and a workspace build would unify that feature back on.

## Usage

```js
import init, { Verifier, version, guestId } from "./pkg/wickra_zk_wasm.js";

await init();

const verifier = new Verifier();

// A proof file produced by the CLI or any native binding.
const outputs = JSON.parse(verifier.command(JSON.stringify({ cmd: "verify", proof })));
// { report_hash, dataset_commitment, guest_id, sharpe, pnl, n_trades }
// -- decoded from the receipt, never read from the file's own copy.

// A verifier who holds the candles checks what the proof is bound to.
const { dataset_commitment } = JSON.parse(verifier.command(JSON.stringify({ cmd: "commit", candles })));
console.log(dataset_commitment === outputs.dataset_commitment);

console.log(version(), guestId()); // the host version and the image id this build pins
```

## Commands

| Command | Payload | Response |
|---------|---------|----------|
| `verify` | `{proof}` | `{report_hash, dataset_commitment, guest_id, sharpe, pnl, n_trades}` |
| `commit` | `{candles}` | `{dataset_commitment}` |
| `version` | — | `{version, guest_id}` |
| `prove` | — | refused: this build carries no prover |

Errors are reported in-band as `{"ok":false,"error":"…"}`, so a page can treat
every response as JSON. A receipt that does not verify -- tampered, for another
guest, or a dev-mode placeholder -- is an error, not a `false`.

## License

Dual-licensed under either [MIT](https://github.com/wickra-lib/wickra-zk/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/wickra-lib/wickra-zk/blob/main/LICENSE-APACHE), at your option.
