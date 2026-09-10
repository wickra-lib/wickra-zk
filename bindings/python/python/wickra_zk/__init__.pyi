from typing import final

__version__: str

@final
class Prover:
    """A prover driven by JSON commands."""

    def __init__(self) -> None: ...
    def command(self, cmd_json: str) -> str:
        """Apply a command JSON and return the response JSON.

        Raises:
            ValueError: if the envelope is malformed or the proof fails.
        """
        ...

    @staticmethod
    def version() -> str: ...
