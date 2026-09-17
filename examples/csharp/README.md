# Wickra ZK examples — C#

Runnable C# examples for the [Wickra ZK C# binding](../../bindings/csharp). The binding consumes the C ABI
library through P/Invoke, so build it once before running anything:

```bash
cargo build -p wickra-zk-c --release
```

## Run

As the CI examples job runs it, from the repository root:

```bash
dotnet run --project examples/csharp/Prove
```

## The examples

| Example | What it does |
|---------|--------------|
| `Prove/Program.cs` | Prove a backtest in zero knowledge from .NET, then verify the proof. |
