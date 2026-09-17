<p align="center">
  <img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp" alt="Wickra" width="100%">
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/ci.svg)](https://github.com/wickra-lib/wickra-zk/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-zk)
[![r-universe](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/r-universe.svg)](https://wickra-lib.r-universe.dev)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/license.svg)](https://github.com/wickra-lib/wickra-zk#license)

# Wickra ZK — R

---

**Wickra ZK — for R. `install.packages("wickrazk", repos = "https://wickra-lib.r-universe.dev")` — over the C ABI via `.Call`, prebuilt library fetched on install.**

R bindings for the `wickra-zk` deterministic zero-knowledge proof core, over its
C ABI hub (`.Call`). Create a stateless prover, drive it with command JSON
(`prove`, `verify`, `version`), read back the response JSON — the
same protocol as the CLI and every other binding.

## Install

From r-universe:

```r
install.packages("wickrazk", repos = "https://wickra-lib.r-universe.dev")
```

The package's `configure` downloads the prebuilt C ABI library for this exact
version from the GitHub release and bundles it, so an ordinary install needs
nothing but a C toolchain (Rtools on Windows) for the thin `.Call` glue layer. To
build against a local checkout instead, point it at the header and library with
the environment variables below.

### Building from this repository (contributors)

The package links the `wickra_zk` C ABI, located out-of-tree via two
environment variables:

```bash
cargo build -p wickra-zk-c --release
export WKZK_INC="$PWD/bindings/c/include"
export WKZK_LIB="$PWD/target/release"
# ensure the shared library is on the loader path at run time
export LD_LIBRARY_PATH="$WKZK_LIB:$LD_LIBRARY_PATH"   # Linux
R CMD INSTALL bindings/r
Rscript bindings/r/tests/run_tests.R
```

On Windows put `wickra_zk.dll` on `PATH`; on macOS use `DYLD_LIBRARY_PATH`.

## Quick start

```r
library(wickrazk)

prover <- wkzk_new()
cmd <- paste0(
  '{"cmd":"prove","spec":{"strategy":{...}},',
  '"candles":[{"time":1,"open":100,"high":101,"low":99,"close":100,"volume":1000}]}'
)
cat(wkzk_command(prover, cmd), "\n")
# {"receipt":…,"journal":{"report_hash":"…","dataset_commitment":"…","guest_id":"…",…},"version":"…"}
cat(wkzk_version(), "\n")
```

### Commands

| Command | Payload | Response |
|---------|---------|----------|
| `prove` | `{spec: {strategy, dataset_commitment?}, candles}` | `{receipt, journal, version}` |
| `commit` | `{candles}` | `{dataset_commitment}` |
| `verify` | `{proof}` | `{report_hash, dataset_commitment, guest_id, sharpe, pnl, n_trades}` |
| `version` | — | `{version, guest_id}` |

The envelope is documented once, in
[docs/ZK.md](https://github.com/wickra-lib/wickra-zk/blob/main/docs/ZK.md#the-command-envelope).
`dataset_commitment` may be omitted from a `prove` spec; the host computes it
from the candles. Proving runs a zkVM: seconds to minutes.

Domain errors are reported in-band as `{"ok":false,"error":"…"}`.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of R's native `.Call` interface over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-zk/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-zk>
- **Docs** (guides, spec reference, cookbook): <https://zk.wickra.org>
- **Runnable example:** [`examples/r/`](https://github.com/wickra-lib/wickra-zk/tree/main/examples/r)

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
