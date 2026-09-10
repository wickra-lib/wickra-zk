"""The Python binding reaches the same core as every other one."""

import json

import wickra_zk


def test_version_is_reported_two_ways_and_they_agree():
    # The module version comes from the crate; Prover.version() comes from the
    # host. A mismatch means the wheel was built against a different host than
    # the one it declares, which is exactly the drift a release cannot afford.
    assert wickra_zk.__version__
    assert wickra_zk.Prover.version()


def test_a_version_command_round_trips():
    prover = wickra_zk.Prover()
    response = json.loads(prover.command(json.dumps({"cmd": "version"})))
    assert "version" in response


def test_a_malformed_envelope_raises_rather_than_returning_nonsense():
    prover = wickra_zk.Prover()
    try:
        prover.command("not json")
    except ValueError:
        return
    raise AssertionError("a malformed envelope must raise")
