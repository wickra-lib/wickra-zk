//! Python bindings for `wickra-zk`, exposed under the `wickra_zk` package.
//!
//! Thin glue over the host's command surface: create a [`Prover`], drive it
//! with a command JSON (`prove`, `verify`, `version`) and read back the
//! response JSON. The same command protocol crosses every binding, so a Python
//! front-end proves against the exact same guest as the native CLI.
//!
//! Proving is not fast and it is not async. `prove` runs a zkVM to completion,
//! which takes seconds to minutes depending on the strategy and on whether the
//! receipt is real or dev-mode, and it holds the GIL for the duration. That is
//! deliberate rather than an oversight: releasing it would invite callers to
//! run several proofs at once on a machine that has one prover's worth of
//! memory, and the failure mode there is an OOM kill rather than a slow answer.

// PyO3 protocol methods take `self` by value/ref regardless of use.
#![allow(clippy::needless_pass_by_value)]

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// A prover driven by JSON commands.
///
/// Holds no state -- the host's command surface is a free function -- but is
/// handle-shaped so every binding in the family reads the same way.
#[pyclass(name = "Prover")]
struct PyProver;

#[pymethods]
impl PyProver {
    /// Create a prover.
    #[new]
    fn new() -> Self {
        Self
    }

    /// Apply a command JSON and return the resulting response JSON.
    fn command(&mut self, cmd_json: &str) -> PyResult<String> {
        wickra_zk_host::command_json(cmd_json)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    /// The library version.
    #[staticmethod]
    fn version() -> &'static str {
        wickra_zk_host::version()
    }
}

/// The native module (`wickra_zk._wickra_zk`).
#[pymodule]
fn _wickra_zk(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add_class::<PyProver>()?;
    Ok(())
}
