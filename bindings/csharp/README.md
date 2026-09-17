<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra ZK — prove a backtest zero-knowledge: on-chain-verifiable performance without revealing the data or the strategy" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/ci.svg)](https://github.com/wickra-lib/wickra-zk/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-zk)
[![NuGet](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/nuget.svg)](https://www.nuget.org/packages/Wickra.Zk)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/license.svg)](https://github.com/wickra-lib/wickra-zk#license)

# Wickra ZK — C#

---

**Wickra ZK — for C#. `dotnet add package Wickra.Zk` — prebuilt native library, no system dependencies.**

The wickra-zk core for .NET, over the C ABI via P/Invoke. The native library ships
inside the NuGet package for every supported runtime identifier, so there is
nothing to install alongside it.

## Install

```bash
dotnet add package Wickra.Zk
```

The native library ships prebuilt per platform under `runtimes/<rid>/native/`,
selected automatically. There is nothing to compile. Targets .NET 8 and later.

### Building from this repository (contributors)

Requires the .NET SDK and a Rust toolchain:

```sh
cargo build -p wickra-zk-c --release
dotnet test bindings/csharp
```

The test project resolves the freshly built native library from `target/release`.

## Quick start

The binding is a thin, faithful surface over the same command boundary every
other binding drives, so a request built here produces the same canonical bytes
it would in Rust, Python or Go.

```csharp
using Wickra.Zk;

using var handle = new Prover();
string response = handle.Command("""{"cmd":"version"}""");
Console.WriteLine(response);
```

`Prover` owns a native handle and implements `IDisposable`; the `using` above is
what releases it. Dropping the reference without disposing leaks the handle
until the finalizer runs.

### What travels with the package

The managed assembly, the native C ABI library for each supported runtime
identifier, and both licence texts. The package declares `MIT OR Apache-2.0`
and carries `LICENSE-MIT` and `LICENSE-APACHE` to match.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of `[LibraryImport]` P/Invoke over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-zk/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-zk>
- **Docs** (guides, spec reference, cookbook): <https://zk.wickra.org>
- **Runnable example:** [`examples/csharp/`](https://github.com/wickra-lib/wickra-zk/tree/main/examples/csharp)

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
