//! Node.js bindings for `wickra-zk` (napi-rs).
//!
//! Thin glue over the host's command surface: create a `Prover`, drive it with
//! a command JSON (`prove`, `verify`, `version`) and read back the response
//! JSON. The same command protocol crosses every binding, so a Node front-end
//! proves against the exact same guest as the native CLI.
//!
//! `command` is synchronous, and for `prove` that means it blocks the event
//! loop for seconds to minutes. That is not an oversight: a zkVM proof is not
//! IO, so moving it to the thread pool would occupy a libuv worker for the same
//! duration and starve everything else that needs one. Run proofs in a worker
//! thread or a separate process if the calling process must stay responsive.

#![allow(missing_debug_implementations)]
// napi exposes owned `String` arguments; the bodies only need to borrow them.
#![allow(clippy::needless_pass_by_value)]
// The napi surface is consumed from JavaScript: `#[must_use]` and `# Errors`
// sections would document a Rust caller that does not exist.
#![allow(clippy::must_use_candidate, clippy::missing_errors_doc)]
// `Prover` holds no state, but its methods take `self` so every binding in
// the family reads the same way.
#![allow(clippy::unused_self)]

use napi::Result;
use napi_derive::napi;

/// Build a napi error from a message.
fn err(message: impl Into<String>) -> napi::Error {
    napi::Error::from_reason(message.into())
}

/// The library version.
#[napi]
pub fn version() -> String {
    wickra_zk_host::version().to_string()
}

/// A prover driven by JSON commands.
///
/// Holds no state -- the host's command surface is a free function -- but is
/// handle-shaped so every binding in the family reads the same way.
#[napi]
pub struct Prover;

#[napi]
impl Prover {
    /// Create a prover.
    #[napi(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    /// Apply a command JSON and return the resulting response JSON.
    #[napi]
    pub fn command(&mut self, cmd_json: String) -> Result<String> {
        wickra_zk_host::command_json(&cmd_json).map_err(|e| err(e.to_string()))
    }

    /// The library version.
    #[napi]
    pub fn version(&self) -> String {
        wickra_zk_host::version().to_string()
    }
}
