# Wickra ZK examples — R

Runnable R examples for the [Wickra ZK R binding](../../bindings/r). The package compiles a thin
`.Call` glue layer against the C ABI library, so build the library and install
the package first (the CI examples job does exactly this):

```bash
cargo build -p wickra-zk-c --release
R CMD INSTALL bindings/r
```

## Run

```bash
Rscript examples/r/prove.R
```

## The examples

| Example | What it does |
|---------|--------------|
| `prove.R` | Prove a backtest in zero knowledge from R, then verify the proof. |
