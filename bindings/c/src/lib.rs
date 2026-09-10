//! The wickra-zk C ABI — the hub every C-capable language links against.
//!
//! The surface is deliberately tiny and JSON-shaped, exactly like
//! [`wickra_zk_host::command_json`]: create a handle, drive it with command
//! JSONs (`prove`, `verify`, `version`) and read back response JSONs, then free
//! the handle. No proof crosses the boundary by value — the handle is opaque and
//! the payloads are always UTF-8 JSON strings.
//!
//! The handle holds nothing. `command_json` is a free function here, so a
//! stateless entry point would have been simpler for C alone — but every other
//! binding in the family is handle-shaped, and a surface that differs per
//! language is one a reader cannot carry between them. `binding-surface` in CI
//! asserts they match.
//!
//! Responses use a caller-owned buffer with a length-out protocol (the classic
//! C two-call idiom), so the caller never frees a callee allocation:
//!
//! 1. Call [`wickra_zk_command`] with `out = NULL`, `cap = 0` to learn the
//!    response length `len` (excluding the terminating NUL).
//! 2. Allocate `len + 1` bytes and call again; the response plus a NUL is
//!    written into `out`.
//!
//! Whenever `len < cap` the response is written immediately, so a
//! sufficiently-large buffer needs only one call. Negative returns are reserved
//! for unusable arguments ([`WICKRA_ZK_ERR_NULL`], [`WICKRA_ZK_ERR_UTF8`]) and
//! caught panics ([`WICKRA_ZK_ERR_PANIC`]); a non-negative return is always the
//! response length. Domain errors — a bad spec, a receipt that does not verify —
//! are *not* negative: they come back in-band as `{"ok":false,"error":...}` JSON
//! in the buffer, which is the same contract every other binding sees.

use core::ffi::{c_char, CStr};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

/// A required pointer argument (`handle` or `cmd_json`) was null.
pub const WICKRA_ZK_ERR_NULL: i32 = -1;
/// `cmd_json` was not valid UTF-8.
pub const WICKRA_ZK_ERR_UTF8: i32 = -2;
/// A panic was caught at the FFI boundary.
pub const WICKRA_ZK_ERR_PANIC: i32 = -3;

/// An opaque handle. Created by [`wickra_zk_new`] and destroyed by
/// [`wickra_zk_free`]; never dereferenced by the caller.
pub struct WickraZk {
    _private: (),
}

/// Read a NUL-terminated C string as `&str`, or `None` on null / bad UTF-8.
///
/// # Safety
/// `ptr` must be null or a valid NUL-terminated C string.
unsafe fn opt_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }.to_str().ok()
}

/// Construct a handle.
///
/// Never fails; free it with [`wickra_zk_free`].
#[no_mangle]
pub extern "C" fn wickra_zk_new() -> *mut WickraZk {
    Box::into_raw(Box::new(WickraZk { _private: () }))
}

/// Destroy a handle. Null is a no-op.
///
/// # Safety
/// `handle` must be null or a handle previously returned by [`wickra_zk_new`]
/// and not already freed.
#[no_mangle]
pub unsafe extern "C" fn wickra_zk_free(handle: *mut WickraZk) {
    if !handle.is_null() {
        drop(unsafe { Box::from_raw(handle) });
    }
}

/// Apply a command JSON and write the response JSON into the caller's buffer.
///
/// Returns the response length in bytes (excluding the terminating NUL), or a
/// negative error code. When the return value `len` satisfies `len < cap`, the
/// response and a trailing NUL have been written to `out`; otherwise `out` is
/// left untouched and the caller should re-call with a `cap` of at least
/// `len + 1`. Pass `out = NULL`, `cap = 0` to query the length without writing.
///
/// # Safety
/// `handle` must be a valid handle; `cmd_json` a valid NUL-terminated C string;
/// `out` either null or a writable buffer of at least `cap` bytes.
#[no_mangle]
pub unsafe extern "C" fn wickra_zk_command(
    handle: *mut WickraZk,
    cmd_json: *const c_char,
    out: *mut c_char,
    cap: usize,
) -> i32 {
    if handle.is_null() || cmd_json.is_null() {
        return WICKRA_ZK_ERR_NULL;
    }
    let Some(cmd) = (unsafe { opt_str(cmd_json) }) else {
        return WICKRA_ZK_ERR_UTF8;
    };

    // Proving runs a zkVM. A panic inside it must not unwind across the FFI
    // boundary -- that is undefined behaviour -- so it is caught here and
    // reported as a code rather than as a crash in the caller's process.
    let response = match catch_unwind(AssertUnwindSafe(|| wickra_zk_host::command_json(cmd))) {
        Ok(Ok(response)) => response,
        // A domain error is not an ABI error: it is an answer. Every other
        // binding sees it as in-band JSON, so C does too.
        Ok(Err(err)) => format!(
            "{{\"ok\":false,\"error\":{}}}",
            json_string(&err.to_string())
        ),
        Err(_) => return WICKRA_ZK_ERR_PANIC,
    };

    let bytes = response.as_bytes();
    let len = bytes.len();
    if len < cap && !out.is_null() {
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), out.cast::<u8>(), len);
            *out.add(len) = 0;
        }
    }
    i32::try_from(len).unwrap_or(i32::MAX)
}

/// The library version as a static NUL-terminated string (do not free).
#[no_mangle]
pub extern "C" fn wickra_zk_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0")
        .as_ptr()
        .cast::<c_char>()
}

/// Encode a string as a JSON string literal (quotes + minimal escaping).
fn json_string(s: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn version_is_a_nul_terminated_string() {
        let ptr = wickra_zk_version();
        let s = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap();
        assert_eq!(s, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn a_handle_round_trips_a_version_command() {
        let handle = wickra_zk_new();
        let cmd = CString::new(r#"{"cmd":"version"}"#).unwrap();

        // First call with no buffer: learn the length.
        let len = unsafe { wickra_zk_command(handle, cmd.as_ptr(), ptr::null_mut(), 0) };
        assert!(len > 0, "expected a length, got {len}");

        // Second call with a buffer of len + 1: the response and a NUL.
        let mut buf = vec![0i8; (len as usize) + 1];
        let again = unsafe {
            wickra_zk_command(handle, cmd.as_ptr(), buf.as_mut_ptr().cast::<c_char>(), buf.len())
        };
        assert_eq!(again, len);
        let text = unsafe { CStr::from_ptr(buf.as_ptr().cast::<c_char>()) }
            .to_str()
            .unwrap();
        assert!(text.contains("version"), "unexpected response: {text}");

        unsafe { wickra_zk_free(handle) };
    }

    #[test]
    fn null_arguments_are_reported_not_dereferenced() {
        let handle = wickra_zk_new();
        let cmd = CString::new(r#"{"cmd":"version"}"#).unwrap();
        assert_eq!(
            unsafe { wickra_zk_command(ptr::null_mut(), cmd.as_ptr(), ptr::null_mut(), 0) },
            WICKRA_ZK_ERR_NULL
        );
        assert_eq!(
            unsafe { wickra_zk_command(handle, ptr::null(), ptr::null_mut(), 0) },
            WICKRA_ZK_ERR_NULL
        );
        unsafe { wickra_zk_free(handle) };
    }

    #[test]
    fn a_domain_error_comes_back_in_band() {
        // An unknown command is an answer, not an ABI failure: the return value
        // stays non-negative and the buffer carries the error.
        let handle = wickra_zk_new();
        let cmd = CString::new(r#"{"cmd":"nope"}"#).unwrap();
        let len = unsafe { wickra_zk_command(handle, cmd.as_ptr(), ptr::null_mut(), 0) };
        assert!(len >= 0, "domain errors must not be negative, got {len}");
        let mut buf = vec![0i8; (len as usize) + 1];
        unsafe {
            wickra_zk_command(handle, cmd.as_ptr(), buf.as_mut_ptr().cast::<c_char>(), buf.len())
        };
        let text = unsafe { CStr::from_ptr(buf.as_ptr().cast::<c_char>()) }
            .to_str()
            .unwrap();
        assert!(text.contains("\"ok\":false"), "unexpected response: {text}");
        unsafe { wickra_zk_free(handle) };
    }

    #[test]
    fn a_short_buffer_leaves_it_untouched_and_reports_the_length() {
        let handle = wickra_zk_new();
        let cmd = CString::new(r#"{"cmd":"version"}"#).unwrap();
        let len = unsafe { wickra_zk_command(handle, cmd.as_ptr(), ptr::null_mut(), 0) };
        let mut buf = vec![0x7fi8; 4];
        let again = unsafe {
            wickra_zk_command(handle, cmd.as_ptr(), buf.as_mut_ptr().cast::<c_char>(), 2)
        };
        assert_eq!(again, len, "a short buffer still reports the true length");
        assert!(buf.iter().all(|b| *b == 0x7f), "the buffer was written to");
        unsafe { wickra_zk_free(handle) };
    }
}
