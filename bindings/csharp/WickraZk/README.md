# Wickra ZK — C&#35;

**Zero-knowledge proofs of backtest performance for .NET, over the Wickra C ABI hub.**

[Wickra ZK](https://github.com/wickra-lib/wickra-zk) runs the deterministic
`wickra-backtest` engine inside a RISC Zero zkVM guest and turns the receipt into
a proof of the report's hash and headline metrics that reveals neither the
candles nor the strategy. This package is the C# binding: it P/Invokes the C ABI
hub and exposes a `Prover` with the same JSON command envelope as every other
binding.

## Install

```bash
dotnet add package Wickra.Zk
```

The package bundles the prebuilt native C ABI library for every supported runtime
(`linux-x64`, `linux-arm64`, `osx-x64`, `osx-arm64`; risc0 has no Windows host) under
`runtimes/<rid>/native/`, resolved automatically at run time.

## Quick start

```csharp
using Wickra.Zk;

using var prover = new Prover();

string cmd = """
{"cmd":"prove","spec":{"strategy":{...}},
 "candles":[{"time":1,"open":100,"high":101,"low":99,"close":100,"volume":1000}]}
""";

string proof = prover.Command(cmd);
// {"receipt":…,"journal":{"report_hash":"…","dataset_commitment":"…","guest_id":"…",…},"version":"…"}

Console.WriteLine(Prover.Version());
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

## Building from this repository (contributors)

```bash
cargo build -p wickra-zk-c --release
dotnet test bindings/csharp/WickraZk.Tests
```

The `DllImportResolver` probes the app directory, the packaged
`runtimes/<rid>/native/`, and the Cargo `target/{release,debug}/` tree, validating
each candidate with a sentinel export so a stale library is rejected.

## License

Dual-licensed under [MIT](https://github.com/wickra-lib/wickra-zk/blob/main/LICENSE-MIT)
or [Apache-2.0](https://github.com/wickra-lib/wickra-zk/blob/main/LICENSE-APACHE), at your option.
