# Wickra ZK examples — Java

Runnable Java examples for the [Wickra ZK Java binding](../../bindings/java). The binding reaches the C ABI
through the Foreign Function & Memory API (JDK 22+), so build the library once
and point the JVM at it with `-Dnative.lib.dir`:

```bash
cargo build -p wickra-zk-c --release
```

## Run

As the CI examples job runs it, from the repository root:

```bash
mvn -f bindings/java/pom.xml -q package -DskipTests
javac -cp bindings/java/target/classes examples/java/Prove.java -d examples/java/out
java --enable-native-access=ALL-UNNAMED  -Dnative.lib.dir="$PWD/target/release"  -cp "bindings/java/target/classes:examples/java/out" Prove examples
```

## The examples

| Example | What it does |
|---------|--------------|
| `Prove.java` | Prove a backtest in zero knowledge from Java, then verify the proof. |
