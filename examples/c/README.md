# Wickra ZK — C / C++ examples

The Wickra ZK C ABI is a single shared/static library plus a generated header
([`bindings/c/include/wickra_zk.h`](../../bindings/c/include/wickra_zk.h)). Any C-capable
language links against the same artifact; these examples show the plain-C path
and, through [`wickra_zk.hpp`](../../bindings/c/include/wickra_zk.hpp), the C++ one.

## Build the library

From the workspace root:

```sh
cargo build -p wickra-zk-c --release
```

This produces, in `target/release/`:

| Platform | Shared library | Link target |
|----------|----------------|-------------|
| Linux    | `libwickra_zk.so`     | `-lwickra_zk` |
| macOS    | `libwickra_zk.dylib`  | `-lwickra_zk` |
| Windows (MSVC) | `wickra_zk.dll` | `wickra_zk.dll.lib` (import lib) |

A static library (`libwickra_zk.a` / `wickra_zk.lib`) is emitted alongside.

## Build and run the examples

### With CMake (portable, used by CI)

```sh
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

### Directly with a compiler

```sh
# Linux / macOS
cc examples/c/prove.c -I bindings/c/include -L target/release -lwickra_zk -lm -o prove
LD_LIBRARY_PATH=target/release ./prove        # macOS: DYLD_LIBRARY_PATH
```

## The examples

| Example | What it does |
|---------|--------------|
| `prove.c` | A minimal C example: prove a (strategy, candles) pair through the wickra-zk |
| `prove.cpp` | A minimal C++ example: prove a (strategy, candles) pair through the wickra-zk C ABI, print the public journal, then verify the proof and assert it holds. |

## Usage shape

Every call follows the same handle discipline: construct from a spec JSON, drive
with command JSON, read the response, free the handle exactly once. `wickra_zk.h` is
the whole contract; the C++ header, where one ships, wraps the handle in a
move-only RAII type. See [`bindings/c/README.md`](../../bindings/c/README.md).
