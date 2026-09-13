"""Prove a backtest in zero knowledge from Python, then verify the proof.

Reads the shared example inputs (``../specs/momentum.json`` and
``../data/BTCUSDT.csv``), proves them through the command envelope -- the
dataset commitment is left to the host -- verifies the proof and prints the
public journal. With ``RISC0_DEV_MODE=1`` the receipt is a fast, unsound
placeholder; without it the real prover runs and this takes minutes.

    pip install wickra-zk
    RISC0_DEV_MODE=1 python prove.py
"""

from __future__ import annotations

import json
from pathlib import Path

import wickra_zk

HERE = Path(__file__).resolve().parent


def load_candles(path: Path) -> list[dict[str, float]]:
    candles = []
    for line in path.read_text().splitlines():
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


def main() -> None:
    strategy = json.loads((HERE / "../specs/momentum.json").read_text())
    candles = load_candles(HERE / "../data/BTCUSDT.csv")

    prover = wickra_zk.Prover()
    proof = json.loads(
        prover.command(json.dumps({"cmd": "prove", "spec": {"strategy": strategy}, "candles": candles}))
    )
    journal = json.loads(prover.command(json.dumps({"cmd": "verify", "proof": proof})))

    print(f"wickra-zk {wickra_zk.Prover.version()}")
    print(f"guest_id: {journal['guest_id']}")
    print(f"report_hash: {journal['report_hash']}")
    print("verify: valid" if journal == proof["journal"] else "verify: INVALID")
    print(json.dumps(journal, indent=2))


if __name__ == "__main__":
    main()
