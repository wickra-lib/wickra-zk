<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra ZK — deterministic zero-knowledge proof for Go" width="100%"></a>
</p>

[![Built on Wickra](https://img.shields.io/badge/built%20on-wickra-3b82f6)](https://github.com/wickra-lib/wickra)
[![Go module](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/go.svg)](https://pkg.go.dev/github.com/wickra-lib/wickra-zk-go)

# Wickra ZK — Go

---

**The deterministic zero-knowledge proof core for Go, over the Wickra C ABI hub via cgo.**

[Wickra ZK](https://github.com/wickra-lib/wickra-zk) folds a `(spec, data)` pair into a deterministic `wickra-backtest` report and a canonical blake3 hash that anyone recomputes byte-for-byte in ten languages. This package is the Go binding: it consumes the C ABI hub through cgo and exposes the stateless `Prover` handle with the same JSON protocol as every other binding.

## Install

Use the published **`wickra-zk-go`** module, which bundles the prebuilt C ABI
library for every platform, so `go get` + `go build` works with no extra steps (a
C compiler is still required, as the binding uses cgo):

```bash
go get github.com/wickra-lib/wickra-zk-go
```

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

	cmd := `{"cmd":"prove","spec":{"strategy":{...},"dataset_ref":"BTCUSDT/1h"},` +
		`"data":{"BTCUSDT":[{"time":1,"open":100,"high":101,"low":99,"close":100,"volume":1000}]}}`

	proof, err := p.Command(cmd)
	if err != nil {
		panic(err)
	}
	fmt.Println(proof) // {"engine_version":"…","inputs_hash":"…","report":…,"report_hash":"…"}
	fmt.Println(wickra.Version())
}
```

## Commands

| Command        | Payload                 | Response                                                |
| -------------- | ----------------------- | ------------------------------------------------------ |
| `prove`        | `{spec, data}`          | `{report, inputs_hash, report_hash, engine_version}`   |
| `verify`       | `{proof, spec, data}`   | `{ok: true, valid: bool}`                              |
| `version`      | —                       | `{engine_version}`                                     |

Errors are reported in-band as `{"ok":false,"error":"…"}`.

`wickra-zk-go` is generated from this directory by the release pipeline: it
mirrors the Go sources, the vendored C ABI header (`include/wickra_zk.h`) and
the prebuilt libraries under `lib/<goos>_<goarch>/` for Linux and macOS (risc0
has no Windows host, so there is no Windows library to mirror).

## Building from this repository (contributors)

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

## License

Dual-licensed under [MIT](https://github.com/wickra-lib/wickra-zk/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/wickra-lib/wickra-zk/blob/main/LICENSE-APACHE), at your option.
