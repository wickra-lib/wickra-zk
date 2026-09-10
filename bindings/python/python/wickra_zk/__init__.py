"""Wickra ZK — prove a backtest without revealing its data or its strategy.

Create a :class:`Prover`, drive it with command JSONs (``prove``, ``verify``,
``version``) and read back response JSONs. The same command protocol crosses
every language binding, so this Python front-end proves against the exact same
zkVM guest as the native CLI.

``prove`` runs a zkVM to completion and takes seconds to minutes. It is a
blocking call and holds the GIL; see the module documentation for why.
"""

from ._wickra_zk import Prover, __version__

__all__ = ["Prover", "__version__"]
