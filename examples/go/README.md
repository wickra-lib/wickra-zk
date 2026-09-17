# Wickra ZK examples — Go

Runnable Go examples for the [Wickra ZK Go binding](../../bindings/go). The binding links against the
prebuilt C ABI library, so build and stage it once before running anything:

```bash
cargo build -p wickra-zk-c --release
mkdir -p target/release bindings/go/lib/linux_amd64
cp "target/$CARGO_BUILD_TARGET/release/libwickra_zk.so" target/release/
cp "target/$CARGO_BUILD_TARGET/release/libwickra_zk.so" bindings/go/lib/linux_amd64/
```

## Run

As the CI examples job runs it, from the repository root:

```bash
cd examples/go && go run .
```

## The examples

| Example | What it does |
|---------|--------------|
| `prove.go` | Prove a backtest in zero knowledge from Go, then verify the proof. |
