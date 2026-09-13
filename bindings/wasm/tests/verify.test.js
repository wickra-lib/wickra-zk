"use strict";

// Tests over the wasm-pack (nodejs target) output, run by the WASM job after
// `wasm-pack build --target nodejs --out-dir pkg-node`:
//
//   real receipt   golden/proofs/<case>.json is a proof the real prover made
//                  for that golden case; the WebAssembly verifier accepts it
//                  and decodes the blessed report_hash and metrics from the
//                  receipt -- the same journal the native verifier returns;
//   binding        the commitment the verifier computes over the case's
//                  candles is the one the journal carries;
//   refusals       a proof whose journal copy lies is refused; a proof for a
//                  tampered receipt is refused; a prove command is refused,
//                  this build carrying no prover; an unknown command is an
//                  in-band error;
//   identity       the version and guest id match the module exports.
//
// The require is hard: a missing build must fail the job, not skip it.

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const { Verifier, version, guestId } = require("../pkg-node/wickra_zk_wasm.js");

const GOLDEN = path.resolve(__dirname, "..", "..", "..", "golden");
const CASES = JSON.parse(fs.readFileSync(path.join(GOLDEN, "cases.json"), "utf8"));
const PROOFS = path.join(GOLDEN, "proofs");

function loadCandles(dataset) {
  const candles = [];
  for (const line of fs.readFileSync(path.join(GOLDEN, "data", `${dataset}.csv`), "utf8").split(/\r?\n/)) {
    const cols = line.split(",").map((c) => c.trim());
    if (cols.length < 6 || !/^\d+$/.test(cols[0])) continue; // header
    candles.push({
      time: Number.parseInt(cols[0], 10),
      open: Number(cols[1]),
      high: Number(cols[2]),
      low: Number(cols[3]),
      close: Number(cols[4]),
      volume: Number(cols[5]),
    });
  }
  return candles;
}

function command(verifier, envelope) {
  return JSON.parse(verifier.command(JSON.stringify(envelope)));
}

const proofs = fs.readdirSync(PROOFS).filter((f) => f.endsWith(".json")).sort();

test("a real proof is committed for at least one golden case", () => {
  assert.ok(proofs.length > 0, "golden/proofs holds at least one real receipt");
});

for (const file of proofs) {
  const name = path.basename(file, ".json");
  const proof = JSON.parse(fs.readFileSync(path.join(PROOFS, file), "utf8"));
  const expected = JSON.parse(fs.readFileSync(path.join(GOLDEN, "expected", `${name}.json`), "utf8"));

  test(`${name}: the real receipt verifies to the blessed journal`, () => {
    const verifier = new Verifier();
    const outputs = command(verifier, { cmd: "verify", proof });
    assert.strictEqual(outputs.ok, undefined, JSON.stringify(outputs));
    assert.strictEqual(outputs.report_hash, expected.report_hash);
    assert.strictEqual(outputs.n_trades, expected.n_trades);
    assert.ok(Math.abs(outputs.sharpe - expected.sharpe) < 1e-8);
    assert.ok(Math.abs(outputs.pnl - expected.pnl) < 1e-8);
    assert.strictEqual(outputs.guest_id, guestId());
    assert.deepStrictEqual(outputs, proof.journal);
  });

  test(`${name}: the journal is bound to the case's candles`, () => {
    const { dataset_commitment } = command(new Verifier(), { cmd: "commit", candles: loadCandles(CASES[name]) });
    assert.strictEqual(dataset_commitment, proof.journal.dataset_commitment);
  });

  test(`${name}: a proof file whose journal lies is refused`, () => {
    const lying = JSON.parse(JSON.stringify(proof));
    lying.journal.report_hash = "f".repeat(64);
    const response = command(new Verifier(), { cmd: "verify", proof: lying });
    assert.strictEqual(response.ok, false);
    assert.match(response.error, /verify/);
  });

  test(`${name}: a tampered receipt is refused`, () => {
    const tampered = JSON.parse(JSON.stringify(proof));
    const bytes = tampered.receipt.journal.bytes;
    bytes[0] = (bytes[0] + 1) % 256;
    const response = command(new Verifier(), { cmd: "verify", proof: tampered });
    assert.strictEqual(response.ok, false);
    assert.match(response.error, /verify/);
  });
}

test("a prove command is refused: this build carries no prover", () => {
  const strategy = JSON.parse(fs.readFileSync(path.join(GOLDEN, "specs", "momentum.json"), "utf8"));
  const response = command(new Verifier(), { cmd: "prove", spec: { strategy }, candles: [] });
  assert.strictEqual(response.ok, false);
  assert.match(response.error, /verifies only/);
});

test("the version and guest id match the module exports", () => {
  const verifier = new Verifier();
  assert.strictEqual(verifier.version(), version());
  assert.strictEqual(verifier.guestId(), guestId());
  const response = command(verifier, { cmd: "version" });
  assert.strictEqual(response.version, version());
  assert.strictEqual(response.guest_id, guestId());
});

test("an unknown command is an in-band error", () => {
  const response = command(new Verifier(), { cmd: "nope" });
  assert.strictEqual(response.ok, false);
  assert.match(response.error, /nope/);
});
