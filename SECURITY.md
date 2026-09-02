# Security Policy

## Supported versions

wickra-zk is in early development. Security fixes are applied to the latest
release, `0.1.0`.

| Version | Supported |
|---------|-----------|
| `0.1.0` (latest) | ✅ |

## Reporting a vulnerability

Please report security issues privately through GitHub's
[private vulnerability reporting](https://github.com/wickra-lib/wickra-zk/security/advisories/new).
Do not open a public issue for a suspected vulnerability.

We aim to acknowledge a report within 72 hours and to provide a remediation
timeline after triage.

## Trust model — what a proof does and does not attest

A wickra-zk receipt proves that the pinned guest program (identified by its
`GUEST_ID`) executed honestly over the prover's inputs and committed the
resulting journal. Two consequences matter for security:

- **A proof reveals only the journal** — the public `report_hash` and metrics.
  The private OHLCV data and strategy internals are guest inputs and are never
  exposed by the receipt.
- **Report soundness relies on the pinned `GUEST_ID`.** A verifier must check the
  receipt against the *expected* image ID. Verifying against an attacker-supplied
  image ID proves nothing about the honest engine — it only proves that *some*
  program ran. Always pin and check the `GUEST_ID` you trust.

A receipt produced in **dev-mode** (`RISC0_DEV_MODE=1`) is a fast fake and is
**not** cryptographically sound; never treat a dev-mode receipt as a real proof.
