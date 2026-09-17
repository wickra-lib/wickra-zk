<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra ZK — prove a backtest zero-knowledge: on-chain-verifiable performance without revealing the data or the strategy" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/ci.svg)](https://github.com/wickra-lib/wickra-zk/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-zk)
[![Go module](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/go.svg)](https://pkg.go.dev/github.com/wickra-lib/wickra-zk-go)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/license.svg)](https://github.com/wickra-lib/wickra-zk#license)

# Wickra ZK — Go

---

**Wickra ZK — for Go. `go get github.com/wickra-lib/wickra-zk-go` — over the C ABI via cgo, prebuilt library bundled in the module.**

[Wickra ZK](https://github.com/wickra-lib/wickra-zk) runs the deterministic `wickra-backtest` engine inside a RISC Zero zkVM guest and turns the receipt into a proof of the report's hash and headline metrics that reveals neither the candles nor the strategy. This package is the Go binding: it consumes the C ABI hub through cgo and exposes the `Prover` handle with the same JSON command envelope as every other binding.

## Install

Use the published **`wickra-zk-go`** module, which bundles the prebuilt C ABI
library for every platform, so `go get` + `go build` works with no extra steps
(a C compiler is still required, as the binding uses cgo):

```bash
go get github.com/wickra-lib/wickra-zk-go
```

`wickra-zk-go` is generated from this directory by the release pipeline: it mirrors
the Go sources, the vendored C ABI header (`include/wickra_zk.h`) and the prebuilt
libraries under `lib/<goos>_<goarch>/`. On Linux/macOS the library path is baked
in via rpath; on Windows the DLL must be discoverable at run time (next to the
executable or on `PATH`).

### Building from this repository (contributors)

This `bindings/go` directory is the development source. To build it directly,
compile the C ABI hub and stage the library into the per-platform directory cgo
links against:

```bash
cargo build -p wickra-zk-c --release
mkdir -p bindings/go/lib/linux_amd64                 # match your GOOS_GOARCH
cp target/release/libwickra_zk.so    bindings/go/lib/linux_amd64/   # Linux
cp target/release/libwickra_zk.dylib bindings/go/lib/darwin_arm64/  # macOS (arm64)
cp target/release/wickra_zk.dll      bindings/go/lib/windows_amd64/ # Windows
```

Then, with the library on the loader path, run `go test ./...` from this directory.

## Quick start

```go
package main

import (
	"fmt"

	wickra "github.com/wickra-lib/wickra-zk-go"
)

func main() {
	p := wickra.New()
	defer p.Close()

	cmd := `{"cmd":"prove","spec":{"strategy":{...}},` +
		`"candles":[{"time":1,"open":100,"high":101,"low":99,"close":100,"volume":1000}]}`

	proof, err := p.Command(cmd)
	if err != nil {
		panic(err)
	}
	fmt.Println(proof) // {"receipt":…,"journal":{"report_hash":"…","dataset_commitment":"…","guest_id":"…",…},"version":"…"}
	fmt.Println(wickra.Version())
}
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

Domain errors are reported in-band as `{"ok":false,"error":"…"}`; the Go
`error` is reserved for ABI-level failures (a null argument, a caught panic).

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of cgo over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-zk/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-zk>
- **Docs** (guides, spec reference, cookbook): <https://zk.wickra.org>
- **Runnable example:** [`examples/go/`](https://github.com/wickra-lib/wickra-zk/tree/main/examples/go)

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
