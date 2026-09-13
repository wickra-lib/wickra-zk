"use strict";

// Operating-mode equivalence: a proof does not depend on who commits the data.
//
// `prove` runs in two operating modes. The caller can leave the dataset
// commitment to the host, which hashes the candles it is handed, or state it
// up front -- the value `commit` returns, or one carried over from a data
// vendor. The journal must not depend on which: same report_hash, same
// metrics, same dataset_commitment, same guest_id, and both proofs verify to
// that journal. Only the receipt bytes may differ between two runs.
//
// Dev-mode receipts are unsound and fast; this tests the binding's transport
// of the envelope, not the proof system.

process.env.RISC0_DEV_MODE = "1";

const { test } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const { Prover } = require("../index.js");

function goldenDir() {
  let dir = __dirname;
  for (let i = 0; i < 8; i++) {
    const candidate = path.join(dir, "golden");
    if (fs.existsSync(path.join(candidate, "cases.json"))) return candidate;
    dir = path.dirname(dir);
  }
  throw new Error("golden/cases.json not found above the test file");
}

const GOLDEN = goldenDir();
const CASES = JSON.parse(fs.readFileSync(path.join(GOLDEN, "cases.json"), "utf8"));

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

function command(prover, envelope) {
  return JSON.parse(prover.command(JSON.stringify(envelope)));
}

for (const name of Object.keys(CASES).sort()) {
  test(`${name}: host-committed and stated-commitment proofs carry one journal`, () => {
    const strategy = JSON.parse(fs.readFileSync(path.join(GOLDEN, "specs", `${name}.json`), "utf8"));
    const candles = loadCandles(CASES[name]);
    const prover = new Prover();

    // Mode 1: the host commits the candles it is handed.
    const host = command(prover, { cmd: "prove", spec: { strategy }, candles });
    // Mode 2: the caller states the commitment up front.
    const commitment = command(prover, { cmd: "commit", candles }).dataset_commitment;
    const stated = command(prover, {
      cmd: "prove",
      spec: { strategy, dataset_commitment: commitment },
      candles,
    });

    assert.deepStrictEqual(stated.journal, host.journal);
    assert.strictEqual(stated.journal.dataset_commitment, commitment);
    assert.deepStrictEqual(command(prover, { cmd: "verify", proof: host }), host.journal);
    assert.deepStrictEqual(command(prover, { cmd: "verify", proof: stated }), host.journal);
  });
}
