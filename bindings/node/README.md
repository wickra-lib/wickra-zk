# wickra-zk — Node.js binding

Prove a backtest in zero knowledge from Node. The same command protocol crosses
every binding, so this front-end proves against the exact same zkVM guest as the
native CLI.

## Install

```sh
npm install wickra-zk
```

The native module ships per platform as an optional dependency; npm picks the
one matching the machine.

## Use

```js
const { Prover } = require("wickra-zk");

const prover = new Prover();
console.log(JSON.parse(prover.command(JSON.stringify({ cmd: "version" }))));
```

## `command` is synchronous, and that is on purpose

For `prove` it blocks the event loop for **seconds to minutes**. Moving it to
the thread pool would not help: a zkVM proof is not IO, so it would occupy a
libuv worker for the same duration and starve everything else that needs one.

Run proofs in a worker thread or a separate process if the calling process has
to stay responsive.

## Errors

A malformed envelope or a failed proof throws, carrying the host's message.
