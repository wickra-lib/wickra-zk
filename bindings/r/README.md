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
  '{"cmd":"prove","spec":{"strategy":{...}},',
  '"candles":[{"time":1,"open":100,"high":101,"low":99,"close":100,"volume":1000}]}'
)
cat(wkzk_command(prover, cmd), "\n")
# {"receipt":…,"journal":{"report_hash":"…","dataset_commitment":"…","guest_id":"…",…},"version":"…"}
cat(wkzk_version(), "\n")
```

## Commands

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

## Build and test from source

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

## License

Dual-licensed under [MIT](https://github.com/wickra-lib/wickra-zk/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/wickra-lib/wickra-zk/blob/main/LICENSE-APACHE), at your option.
