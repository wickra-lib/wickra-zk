<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra ZK — prove a backtest zero-knowledge: on-chain-verifiable performance without revealing the data or the strategy" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/ci.svg)](https://github.com/wickra-lib/wickra-zk/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-zk)
[![npm](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/npm.svg)](https://www.npmjs.com/package/wickra-zk-wasm)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/license.svg)](https://github.com/wickra-lib/wickra-zk#license)

# Wickra ZK — WASM

---

**Wickra ZK — for WASM. `npm install wickra-zk-wasm` — pure WebAssembly, runs anywhere a modern JS engine does.**

The verifying side of `wickra-zk`, compiled to WebAssembly with wasm-bindgen: a
browser receives a proof -- a RISC Zero receipt and its public journal -- and
checks it against the guest this build pins, without a prover, without the
candles and without the strategy.

Create a `Verifier`, drive it with a command JSON and read back the response
JSON: the same envelope every other binding speaks, minus `prove`. Proving runs
a zkVM and belongs to a native process; a browser verifies what it is handed.

## Install

```bash
npm install wickra-zk-wasm
```

### Building from this repository (contributors)

```bash
wasm-pack build --target web            # pkg/  -- what ships
wasm-pack build --target nodejs --out-dir pkg-node   # for the tests
cargo test                              # the crate's own unit tests, natively
```

This crate is its own workspace: the host it wraps is built without its
`prove` feature, and a workspace build would unify that feature back on.

## Quick start

```js
import init, { Verifier, version, guestId } from "wickra-zk-wasm";

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

### Commands

| Command | Payload | Response |
|---------|---------|----------|
| `verify` | `{proof}` | `{report_hash, dataset_commitment, guest_id, sharpe, pnl, n_trades}` |
| `commit` | `{candles}` | `{dataset_commitment}` |
| `version` | — | `{version, guest_id}` |
| `prove` | — | refused: this build carries no prover |

Errors are reported in-band as `{"ok":false,"error":"…"}`, so a page can treat
every response as JSON. A receipt that does not verify -- tampered, for another
guest, or a dev-mode placeholder -- is an error, not a `false`.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of wasm-bindgen, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-zk/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-zk>
- **Docs** (guides, spec reference, cookbook): <https://zk.wickra.org>
- **Runnable example:** [`examples/wasm/`](https://github.com/wickra-lib/wickra-zk/tree/main/examples/wasm)

Wickra ZK ships native bindings for Python, Node.js, WASM and Rust, plus a C ABI hub that any
C-capable language (C, C++, C#, Go, Java, R) links against — all forwarding to the
same data-driven, `unsafe`-forbidden Rust core.

## Security

Found a security issue? **Please don't open a public issue.** Report it privately
via the repository's *Security* tab (*"Report a vulnerability"*) or email
**support@wickra.org** with a subject line starting `[wickra security]`. Full
policy: <https://github.com/wickra-lib/wickra-zk/blob/main/SECURITY.md>.

## Disclaimer

This software is provided for research and educational purposes. It is not
financial advice. A zero-knowledge proof attests only to the honest execution of
the pinned guest program over the prover's inputs; it makes no claim about the
quality, provenance, or future performance of a trading strategy.

## License

Licensed under either of [Apache-2.0](https://github.com/wickra-lib/wickra-zk/blob/main/LICENSE-APACHE)
or [MIT](https://github.com/wickra-lib/wickra-zk/blob/main/LICENSE-MIT) at your option.
