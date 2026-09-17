<p align="center">
  <img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp" alt="Wickra" width="100%">
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/ci.svg)](https://github.com/wickra-lib/wickra-zk/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-zk)
[![Maven Central](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/maven.svg)](https://central.sonatype.com/artifact/org.wickra/wickra-zk)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-zk/license.svg)](https://github.com/wickra-lib/wickra-zk#license)

# Wickra ZK — Java

---

**Wickra ZK — for Java. `org.wickra:wickra-zk` — prebuilt native library inside the jar, no JNI, no system dependencies.**

JVM bindings for the `wickra-zk` deterministic zero-knowledge proof core over its
C ABI hub (FFM / Panama, `java.lang.foreign`). Create a stateless `Prover`, drive
it with command JSON (`prove`, `verify`, `version`) and read back
the response JSON — the same protocol as every other binding.

## Requirements

- Java 22+ (the Foreign Function & Memory API is stable since 22).
- Run with `--enable-native-access=ALL-UNNAMED`.
- The native library (`wickra_zk`) must be resolvable — either on the library
  path or via the `native.lib.dir` system property pointing at the directory that
  holds `libwickra_zk.{so,dylib}` (Linux and macOS; risc0 has no Windows host).

## Install

Maven:

```xml
<dependency>
  <groupId>org.wickra</groupId>
  <artifactId>wickra-zk</artifactId>
  <version>0.1.1</version>
</dependency>
```

Gradle:

```kotlin
implementation("org.wickra:wickra-zk:0.1.1")
```

The native library ships prebuilt per platform inside the jar and is
extracted automatically on first use. There is nothing to compile.

### Building from this repository (contributors)

```bash
cargo build -p wickra-zk-c
mvn -f bindings/java/pom.xml test
```

## Quick start

```java
import org.wickra.zk.Prover;

try (Prover prover = new Prover()) {
    String cmd = """
        {"cmd":"prove","spec":{"strategy":{...}},
        "candles":[{"time":1,"open":100,"high":101,"low":99,"close":100,"volume":1000}]}""";
    System.out.println(prover.command(cmd));
    // {"receipt":…,"journal":{"report_hash":"…","dataset_commitment":"…","guest_id":"…",…},"version":"…"}
}
System.out.println(Prover.version());
```

### API

| Member | Description |
|--------|-------------|
| `new Prover()` | Create a stateless prover. |
| `String command(String cmdJson)` | Apply a command JSON, return the response JSON. |
| `static String version()` | The library version. |
| `close()` | Free the native handle (via `AutoCloseable`). |

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

Domain errors are reported in-band as `{"ok":false,"error":"…"}`.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of the Java Foreign Function & Memory API over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-zk/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-zk>
- **Docs** (guides, spec reference, cookbook): <https://zk.wickra.org>
- **Runnable example:** [`examples/java/`](https://github.com/wickra-lib/wickra-zk/tree/main/examples/java)

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
