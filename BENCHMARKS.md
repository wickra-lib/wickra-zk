# Benchmarks

Placeholder — real numbers are filled in from the `wickra-zk-bench` criterion
suite (P-ZK-6) once the host and guest are in place.

The benchmarks will cover:

- **Execution (dev-mode)** — running the guest to produce the journal without a
  real proof, i.e. the cost of the backtest inside the zkVM.
- **Proving (production)** — generating a real receipt (`sample-size = 10`,
  `#[ignore]` by default; run in the nightly `prove.yml` workflow because it is
  slow and memory-hungry).
- **Verification** — checking a receipt against the pinned `GUEST_ID`.

Numbers are measured on the CI reference runner and reported as median. Proving
time and memory dominate; execution and verification are comparatively cheap.
