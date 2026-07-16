#![no_main]
//! `ZkSpec::from_json` must never panic on arbitrary input.

use libfuzzer_sys::fuzz_target;
use wickra_zk_host::ZkSpec;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = ZkSpec::from_json(text);
    }
});
