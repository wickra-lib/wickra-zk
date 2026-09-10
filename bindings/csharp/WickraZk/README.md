# Wickra ZK — C&#35;

**The deterministic zero-knowledge proof core for .NET over the Wickra C ABI hub.**

[Wickra ZK](https://github.com/wickra-lib/wickra-zk) folds a `(spec, data)`
pair into a deterministic `wickra-backtest` report and a canonical blake3 hash that
anyone recomputes byte-for-byte in ten languages. This package is the C# binding:
it P/Invokes the C ABI hub and exposes a stateless `Prover` with the same JSON
protocol as every other binding.

## Install

```bash
dotnet add package Wickra.Proof
```

The package bundles the prebuilt native C ABI library for every supported runtime
(`win-x64`, `win-arm64`, `linux-x64`, `linux-arm64`, `osx-x64`, `osx-arm64`) under
`runtimes/<rid>/native/`, resolved automatically at run time.

## Quick start

```csharp
using Wickra.Proof;

using var prover = new Prover();

string cmd = """
{"cmd":"prove","spec":{"strategy":{...},"dataset_ref":"BTCUSDT/1h"},
 "data":{"BTCUSDT":[{"time":1,"open":100,"high":101,"low":99,"close":100,"volume":1000}]}}
""";

string proof = prover.Command(cmd);
// {"engine_version":"…","inputs_hash":"…","report":…,"report_hash":"…"}

Console.WriteLine(Prover.Version());
```

## Commands

| Command        | Payload                 | Response                                              |
| -------------- | ----------------------- | ---------------------------------------------------- |
| `prove`        | `{spec, data}`          | `{report, inputs_hash, report_hash, engine_version}` |
| `verify`       | `{proof, spec, data}`   | `{ok: true, valid: bool}`                            |
| `version`      | —                       | `{engine_version}`                                   |

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
