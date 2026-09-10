# Wickra ZK — R

R bindings for the `wickra-zk` deterministic zero-knowledge proof core, over its
C ABI hub (`.Call`). Create a stateless prover, drive it with command JSON
(`prove`, `verify`, `version`), read back the response JSON — the
same protocol as the CLI and every other binding.

## Usage

```r
library(wickrazk)

prover <- wkzk_new()
cmd <- paste0(
  '{"cmd":"prove","spec":{"strategy":{...},"dataset_ref":"BTCUSDT/1h"},',
  '"data":{"BTCUSDT":[{"time":1,"open":100,"high":101,"low":99,"close":100,"volume":1000}]}}'
)
cat(wkzk_command(prover, cmd), "\n")
# {"engine_version":"…","inputs_hash":"…","report":…,"report_hash":"…"}
cat(wkzk_version(), "\n")
```

## Commands

| Command | Payload | Response |
|---------|---------|----------|
| `prove` | `{spec, data}` | `{report, inputs_hash, report_hash, engine_version}` |
| `verify` | `{proof, spec, data}` | `{ok: true, valid: bool}` |
| `version` | — | `{engine_version}` |

Domain errors are reported in-band as `{"ok":false,"error":"…"}`.

## Build and test from source

The package links the `wickra_zk` C ABI, located out-of-tree via two
environment variables:

```bash
cargo build -p wickra-zk-c --release
export WKPROOF_INC="$PWD/bindings/c/include"
export WKPROOF_LIB="$PWD/target/release"
# ensure the shared library is on the loader path at run time
export LD_LIBRARY_PATH="$WKPROOF_LIB:$LD_LIBRARY_PATH"   # Linux
R CMD INSTALL bindings/r
Rscript bindings/r/tests/run_tests.R
```

On Windows put `wickra_zk.dll` on `PATH`; on macOS use `DYLD_LIBRARY_PATH`.

## License

Dual-licensed under [MIT](https://github.com/wickra-lib/wickra-zk/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/wickra-lib/wickra-zk/blob/main/LICENSE-APACHE), at your option.
