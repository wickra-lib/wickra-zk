//! WebAssembly binding for `wickra-zk` (wasm-bindgen): the verifying side.
//!
//! A browser receives a proof -- a receipt and its public journal -- and
//! checks it against the pinned guest, without a prover, without the candles
//! and without the strategy. Create a `Verifier`, drive it with a command JSON
//! and read back the response JSON: the same envelope every other binding
//! speaks, minus `prove`.
//!
//! - `{"cmd":"verify","proof":<ZkProof>}` -> the public outputs decoded from
//!   the receipt, or an in-band error when the receipt does not hold.
//! - `{"cmd":"commit","candles":[...]}` -> the dataset commitment a verifier
//!   who holds the candles compares with the journal's.
//! - `{"cmd":"version"}` -> `{version, guest_id}`: the host version and the
//!   image id this build pins.
//! - `{"cmd":"prove", ...}` -> an in-band error: this build carries no prover.
//!
//! Errors are in-band, `{"ok":false,"error":"..."}`, the way the C ABI reports
//! them, so a page can treat every response as JSON.

// The surface is consumed from JavaScript: `#[must_use]` would document a Rust
// caller that does not exist.
#![allow(clippy::must_use_candidate)]

use wasm_bindgen::prelude::*;

/// A verifier driven by JSON commands. Holds no state; handle-shaped so the
/// family's bindings read alike.
#[wasm_bindgen]
pub struct Verifier;

#[wasm_bindgen]
impl Verifier {
    /// Create a verifier.
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Verifier {
        Self
    }

    /// Apply a command JSON and return the response JSON. Errors are in-band.
    #[allow(clippy::unused_self)]
    pub fn command(&self, cmd_json: &str) -> String {
        match wickra_zk_host::command_json(cmd_json) {
            Ok(response) => response,
            Err(err) => in_band_error(&err.to_string()),
        }
    }

    /// The host version this build carries.
    #[wasm_bindgen(js_name = version)]
    #[allow(clippy::unused_self)]
    pub fn instance_version(&self) -> String {
        wickra_zk_host::version().to_string()
    }

    /// The image id this build pins: a receipt for any other guest is refused.
    #[wasm_bindgen(js_name = guestId)]
    #[allow(clippy::unused_self)]
    pub fn guest_id(&self) -> String {
        wickra_zk_host::guest_id()
    }
}

/// The host version this build carries.
#[wasm_bindgen]
pub fn version() -> String {
    wickra_zk_host::version().to_string()
}

/// The image id this build pins.
#[wasm_bindgen(js_name = guestId)]
pub fn guest_id() -> String {
    wickra_zk_host::guest_id()
}

/// `{"ok":false,"error":<message as a JSON string>}`.
fn in_band_error(message: &str) -> String {
    let mut out = String::from("{\"ok\":false,\"error\":\"");
    for c in message.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                use std::fmt::Write as _;
                write!(out, "\\u{:04x}", c as u32).expect("write to a String");
            }
            c => out.push(c),
        }
    }
    out.push_str("\"}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_are_in_band_json() {
        let response = Verifier::new().command("{\"cmd\":\"nope\"}");
        assert!(
            response.starts_with("{\"ok\":false,\"error\":\""),
            "{response}"
        );
        assert!(response.contains("nope"));
    }

    #[test]
    fn a_prove_command_is_refused_in_band() {
        // A well-formed prove request, so what refuses it is the missing
        // prover and not the parser.
        let strategy = include_str!("../../../golden/specs/momentum.json");
        let response = Verifier::new().command(&format!(
            "{{\"cmd\":\"prove\",\"spec\":{{\"strategy\":{strategy}}},\"candles\":[]}}"
        ));
        assert!(response.contains("verifies only"), "{response}");
    }

    #[test]
    fn version_names_the_pinned_guest() {
        assert_eq!(Verifier::new().guest_id(), guest_id());
        assert_eq!(Verifier::new().instance_version(), version());
        let response = Verifier::new().command("{\"cmd\":\"version\"}");
        assert!(response.contains(&guest_id()), "{response}");
    }

    #[test]
    fn escaping_covers_quotes_backslashes_and_control_characters() {
        assert_eq!(
            in_band_error("a\"b\\c\nd\u{1}"),
            "{\"ok\":false,\"error\":\"a\\\"b\\\\c\\nd\\u0001\"}"
        );
    }
}
