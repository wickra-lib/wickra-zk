// The Node binding reaches the same core as every other one.
import assert from "node:assert/strict";
import test from "node:test";

const { Prover, version } = require("../index.js");

test("the version is reported two ways and they agree", () => {
  assert.ok(version());
  const p = new Prover();
  assert.equal(p.version(), version());
});

test("a version command round trips", () => {
  const p = new Prover();
  const response = JSON.parse(p.command(JSON.stringify({ cmd: "version" })));
  assert.ok("version" in response);
});

test("a malformed envelope throws rather than returning nonsense", () => {
  const p = new Prover();
  assert.throws(() => p.command("not json"));
});
