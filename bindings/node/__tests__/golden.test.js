"use strict";

// Cross-language golden parity through the command envelope, in dev-mode.
//
// Each golden case is proven through the binding without a stated commitment,
// and the journal must carry the blessed report_hash and metrics -- the same
// determinism chain crates/wickra-zk-host/tests/golden.rs pins natively. The
// proof then verifies to the same journal, a proof file whose journal disagrees
// with its receipt is refused, and a stated commitment that is not the hash of
// the candles is refused before the zkVM runs.
//
// Dev-mode receipts are unsound and fast; this tests the binding's transport
// of the envelope, not the proof system. The real prover runs nightly.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");

process.env.RISC0_DEV_MODE = "1";

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
  test(`golden ${name} proves to the blessed journal`, () => {
    const strategy = JSON.parse(fs.readFileSync(path.join(GOLDEN, "specs", `${name}.json`), "utf8"));
    const expected = JSON.parse(fs.readFileSync(path.join(GOLDEN, "expected", `${name}.json`), "utf8"));
    const candles = loadCandles(CASES[name]);
    const prover = new Prover();

    const commitment = command(prover, { cmd: "commit", candles }).dataset_commitment;
    const proof = command(prover, { cmd: "prove", spec: { strategy }, candles });
    const journal = proof.journal;

    assert.equal(journal.report_hash, expected.report_hash);
    assert.equal(journal.n_trades, expected.n_trades);
    assert.ok(Math.abs(journal.sharpe - expected.sharpe) < 1e-8);
    assert.ok(Math.abs(journal.pnl - expected.pnl) < 1e-8);
    assert.equal(journal.dataset_commitment, commitment);
    assert.equal(journal.guest_id, command(prover, { cmd: "version" }).guest_id);

    assert.deepEqual(command(prover, { cmd: "verify", proof }), journal);

    const lying = JSON.parse(JSON.stringify(proof));
    lying.journal.report_hash = "f".repeat(64);
    assert.throws(() => command(prover, { cmd: "verify", proof: lying }), /verify/);
  });
}

test("a stated commitment is held to", () => {
  const name = Object.keys(CASES).sort()[0];
  const strategy = JSON.parse(fs.readFileSync(path.join(GOLDEN, "specs", `${name}.json`), "utf8"));
  const candles = loadCandles(CASES[name]);
  assert.throws(
    () =>
      command(new Prover(), {
        cmd: "prove",
        spec: { strategy, dataset_commitment: "0".repeat(64) },
        candles,
      }),
    /commitment mismatch/,
  );
});
