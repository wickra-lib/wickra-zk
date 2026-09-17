<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra ZK — prove a backtest zero-knowledge: on-chain-verifiable performance without revealing the data or the strategy" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/ci.svg)](https://github.com/wickra-lib/wickra-zk/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-zk)
[![PyPI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/pypi.svg)](https://pypi.org/project/wickra-zk/)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/license.svg)](https://github.com/wickra-lib/wickra-zk#license)

# Wickra ZK — Python

---

**Wickra ZK — for Python. `pip install wickra-zk` — prebuilt wheels for Linux, macOS and Windows, nothing to compile.**

Prove a backtest in zero knowledge from Python. The same command protocol
crosses every binding, so this front-end proves against the exact same zkVM
guest as the native CLI.

## Install

```bash
pip install wickra-zk
```

Pre-built wheels ship for Linux, macOS and Windows — there is nothing to
compile and no C library to track down.

## Quick start

```python
import json
import wickra_zk

prover = wickra_zk.Prover()
print(json.loads(prover.command(json.dumps({"cmd": "version"}))))
```

`prove` runs a zkVM to completion: **seconds to minutes**, depending on the
strategy and on whether the receipt is real or dev-mode. It is a blocking call
and holds the GIL for the duration.

That is deliberate. Releasing the GIL would invite callers to run several proofs
concurrently on a machine that has one prover's worth of memory, and the failure
mode there is an OOM kill rather than a slow answer. Run proofs in separate
processes if you need concurrency, so each one gets its own memory budget.

### Errors

A malformed envelope or a failed proof raises `ValueError` carrying the host's
message. That is the one place this binding differs from the C ABI, which
reports the same conditions in-band as `{"ok":false,...}` — a Python caller
expects an exception, and the surface check accounts for the difference.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of PyO3, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-zk/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-zk>
- **Docs** (guides, spec reference, cookbook): <https://zk.wickra.org>
- **Runnable example:** [`examples/python/`](https://github.com/wickra-lib/wickra-zk/tree/main/examples/python)

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
