# wickra-zk — Python binding

Prove a backtest in zero knowledge from Python. The same command protocol
crosses every binding, so this front-end proves against the exact same zkVM
guest as the native CLI.

## Install

```sh
pip install wickra-zk
```

## Use

```python
import json
import wickra_zk

prover = wickra_zk.Prover()
print(json.loads(prover.command(json.dumps({"cmd": "version"}))))
```

`prove` runs a zkVM to completion: **seconds to minutes**, depending on the
strategy and on whether the receipt is real or dev-mode. It is a blocking call
and holds the GIL for the duration.

That is deliberate. Releasing the GIL would invite callers to run several proofs
concurrently on a machine that has one prover's worth of memory, and the failure
mode there is an OOM kill rather than a slow answer. Run proofs in separate
processes if you need concurrency, so each one gets its own memory budget.

## Errors

A malformed envelope or a failed proof raises `ValueError` carrying the host's
message. That is the one place this binding differs from the C ABI, which
reports the same conditions in-band as `{"ok":false,...}` — a Python caller
expects an exception, and the surface check accounts for the difference.
