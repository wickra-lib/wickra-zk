# Wickra ZK — Java

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

## Usage

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

## API

| Member | Description |
|--------|-------------|
| `new Prover()` | Create a stateless prover. |
| `String command(String cmdJson)` | Apply a command JSON, return the response JSON. |
| `static String version()` | The library version. |
| `close()` | Free the native handle (via `AutoCloseable`). |

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
cargo build -p wickra-zk-c
mvn -f bindings/java/pom.xml test
```

## License

`MIT OR Apache-2.0`.
