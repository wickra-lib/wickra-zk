"""Operating-mode equivalence: a proof does not depend on who commits the data.

``prove`` runs in two operating modes. The caller can leave the dataset
commitment to the host, which hashes the candles it is handed, or state it up
front -- the value ``commit`` returns, or one carried over from a data vendor.
The journal must not depend on which: same ``report_hash``, same metrics,
same ``dataset_commitment``, same ``guest_id``, and both proofs verify to that
journal. Only the receipt bytes may differ between two runs.

Dev-mode receipts are unsound and fast; this tests the binding's transport of
the envelope, not the proof system.
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


def test_host_committed_and_stated_commitment_prove_alike() -> None:
    for case in sorted(CASES):
        strategy = json.loads((GOLDEN / "specs" / f"{case}.json").read_text())
        candles = load_candles(CASES[case])
        prover = wickra_zk.Prover()

        # Mode 1: the host commits the candles it is handed.
        host = command(prover, cmd="prove", spec={"strategy": strategy}, candles=candles)
        # Mode 2: the caller states the commitment up front.
        commitment = command(prover, cmd="commit", candles=candles)["dataset_commitment"]
        stated = command(
            prover,
            cmd="prove",
            spec={"strategy": strategy, "dataset_commitment": commitment},
            candles=candles,
        )

        assert stated["journal"] == host["journal"], case
        assert stated["journal"]["dataset_commitment"] == commitment, case
        assert command(prover, cmd="verify", proof=host) == host["journal"], case
        assert command(prover, cmd="verify", proof=stated) == host["journal"], case
