<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra ZK — prove a backtest zero-knowledge: on-chain-verifiable performance without revealing the data or the strategy" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/ci.svg)](https://github.com/wickra-lib/wickra-zk/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-zk)
[![npm](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/npm.svg)](https://www.npmjs.com/package/wickra-zk)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/license.svg)](https://github.com/wickra-lib/wickra-zk#license)

# Wickra ZK — Node.js

---

**Wickra ZK — for Node.js. `npm install wickra-zk` — prebuilt native binary, no system dependencies.**

Prove a backtest in zero knowledge from Node. The same command protocol crosses
every binding, so this front-end proves against the exact same zkVM guest as the
native CLI.

## Install

```bash
npm install wickra-zk
```

The native addon ships as a prebuilt binary per platform (Linux, macOS,
Windows — x64 and arm64), selected automatically through optional
dependencies. There is nothing to compile.

The native module ships per platform as an optional dependency; npm picks the
one matching the machine.

## Quick start

```js
const { Prover } = require("wickra-zk");

const prover = new Prover();
console.log(JSON.parse(prover.command(JSON.stringify({ cmd: "version" }))));
```

### `command` is synchronous, and that is on purpose

For `prove` it blocks the event loop for **seconds to minutes**. Moving it to
the thread pool would not help: a zkVM proof is not IO, so it would occupy a
libuv worker for the same duration and starve everything else that needs one.

Run proofs in a worker thread or a separate process if the calling process has
to stay responsive.

### Errors

A malformed envelope or a failed proof throws, carrying the host's message.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of napi-rs, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-zk/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-zk>
- **Docs** (guides, spec reference, cookbook): <https://zk.wickra.org>
- **Runnable example:** [`examples/node/`](https://github.com/wickra-lib/wickra-zk/tree/main/examples/node)

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
