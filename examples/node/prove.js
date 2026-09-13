"use strict";

// Prove a backtest in zero knowledge from Node.js, then verify the proof.
//
// Reads the shared example inputs (../specs/momentum.json and
// ../data/BTCUSDT.csv), proves them through the command envelope -- the
// dataset commitment is left to the host -- verifies the proof and prints the
// public journal. With RISC0_DEV_MODE=1 the receipt is a fast, unsound
// placeholder; without it the real prover runs and this takes minutes.
//
//   npm install
//   RISC0_DEV_MODE=1 node prove.js

const fs = require("node:fs");
const path = require("node:path");
const { Prover, version } = require("wickra-zk");

function loadCandles(file) {
  const candles = [];
  for (const line of fs.readFileSync(file, "utf8").split(/\r?\n/)) {
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

const strategy = JSON.parse(fs.readFileSync(path.join(__dirname, "../specs/momentum.json"), "utf8"));
const candles = loadCandles(path.join(__dirname, "../data/BTCUSDT.csv"));

const prover = new Prover();
const proof = JSON.parse(prover.command(JSON.stringify({ cmd: "prove", spec: { strategy }, candles })));
const journal = JSON.parse(prover.command(JSON.stringify({ cmd: "verify", proof })));

console.log(`wickra-zk ${version()}`);
console.log(`guest_id: ${journal.guest_id}`);
console.log(`report_hash: ${journal.report_hash}`);
console.log(JSON.stringify(journal) === JSON.stringify(proof.journal) ? "verify: valid" : "verify: INVALID");
console.log(JSON.stringify(journal, null, 2));
