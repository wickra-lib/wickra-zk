"""Cross-language golden parity through the command envelope, in dev-mode.

Each golden case is proven through the binding without a stated commitment,
and the journal must carry the blessed ``report_hash`` and metrics -- the same
determinism chain ``crates/wickra-zk-host/tests/golden.rs`` pins natively. The
proof then verifies to the same journal, a proof file whose journal disagrees
with its receipt is refused, and a stated commitment that is not the hash of
the candles is refused before the zkVM runs.

Dev-mode receipts are unsound and fast; this tests the binding's transport of
the envelope, not the proof system. The real prover runs nightly in prove.yml.
"""

from __future__ import annotations

import json
import os
from pathlib import Path

os.environ["RISC0_DEV_MODE"] = "1"

import wickra_zk  # noqa: E402


def _golden_dir() -> Path:
    for parent in Path(__file__).resolve().parents:
        candidate = parent / "golden"
        if (candidate / "cases.json").is_file():
            return candidate
    raise FileNotFoundError("golden/cases.json not found above the test file")


GOLDEN = _golden_dir()
CASES = json.loads((GOLDEN / "cases.json").read_text())


def load_candles(dataset: str) -> list[dict[str, float]]:
    candles = []
    for line in (GOLDEN / "data" / f"{dataset}.csv").read_text().splitlines():
        cols = [c.strip() for c in line.split(",")]
        if len(cols) < 6 or not cols[0].isdigit():
            continue  # header
        candles.append(
            {
                "time": int(cols[0]),
                "open": float(cols[1]),
                "high": float(cols[2]),
                "low": float(cols[3]),
                "close": float(cols[4]),
                "volume": float(cols[5]),
            }
        )
    return candles


def command(prover: wickra_zk.Prover, **envelope):
    return json.loads(prover.command(json.dumps(envelope)))


def raises(message: str, **envelope) -> None:
    """The envelope must be refused with an in-band error naming `message`."""
    try:
        command(wickra_zk.Prover(), **envelope)
    except ValueError as err:
        assert message in str(err), str(err)
        return
    raise AssertionError(f"expected a refusal naming {message!r}")


def test_golden_cases_prove_to_the_blessed_journal() -> None:
    assert CASES, "golden/cases.json names at least one case"
    for case in sorted(CASES):
        golden_case(case)


def golden_case(case: str) -> None:
    strategy = json.loads((GOLDEN / "specs" / f"{case}.json").read_text())
    expected = json.loads((GOLDEN / "expected" / f"{case}.json").read_text())
    candles = load_candles(CASES[case])
    prover = wickra_zk.Prover()

    commitment = command(prover, cmd="commit", candles=candles)["dataset_commitment"]
    proof = command(prover, cmd="prove", spec={"strategy": strategy}, candles=candles)
    journal = proof["journal"]

    assert journal["report_hash"] == expected["report_hash"]
    assert journal["n_trades"] == expected["n_trades"]
    assert abs(journal["sharpe"] - expected["sharpe"]) < 1e-8
    assert abs(journal["pnl"] - expected["pnl"]) < 1e-8
    assert journal["dataset_commitment"] == commitment
    assert journal["guest_id"] == command(prover, cmd="version")["guest_id"]

    assert command(prover, cmd="verify", proof=proof) == journal

    lying = json.loads(json.dumps(proof))
    lying["journal"]["report_hash"] = "f" * 64
    raises("verify", cmd="verify", proof=lying)


def test_a_stated_commitment_is_held_to() -> None:
    case = sorted(CASES)[0]
    strategy = json.loads((GOLDEN / "specs" / f"{case}.json").read_text())
    candles = load_candles(CASES[case])
    raises(
        "commitment mismatch",
        cmd="prove",
        spec={"strategy": strategy, "dataset_commitment": "0" * 64},
        candles=candles,
    )
