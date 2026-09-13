//! Build the detached RISC-V guest crate for the builder to export.
//!
//! `risc0_build::embed_methods()` compiles `../methods/guest` for the
//! `riscv32im-risc0-zkvm-elf` target and writes `$OUT_DIR/methods.rs`, which
//! `src/main.rs` includes: `WICKRA_ZK_GUEST_ELF` (the program bytes) and
//! `WICKRA_ZK_GUEST_ID` (its image ID). The builder then writes both into
//! `guest/methods`, where they are committed -- see `src/main.rs`.

/// Environment the host build was invoked with that must not reach the guest
/// build. The guest is compiled by risc0's own toolchain for its own target;
/// a wrapper or a flag meant for the host breaks it: cargo-llvm-cov's
/// `RUSTC_WRAPPER` injects `-C instrument-coverage`, and the risc0 toolchain
/// carries no profiler runtime (`can't find crate for profiler_builtins`);
/// clippy's `RUSTC_WORKSPACE_WRAPPER` is clippy-driver, which has no std for
/// `riscv32im-risc0-zkvm-elf`. risc0-build drops the `CARGO_*` variables from
/// the cargo it spawns and nothing else; these are the ones it leaves.
const HOST_ONLY: &[&str] = &[
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "RUSTFLAGS",
    "RUSTDOCFLAGS",
    "LLVM_PROFILE_FILE",
];

fn main() {
    for key in HOST_ONLY {
        std::env::remove_var(key);
    }
    // The guest resolves its dependencies against its own committed
    // Cargo.lock, and only against it: a transitive crate that moved would
    // otherwise change the ELF -- and the image id -- with no change to any
    // source in this repository.
    std::env::set_var("RISC0_BUILD_LOCKED", "1");
    let cov: Vec<String> = std::env::vars()
        .map(|(key, _)| key)
        .filter(|key| key.starts_with("__CARGO_LLVM_COV"))
        .collect();
    for key in cov {
        std::env::remove_var(key);
    }
    risc0_build::embed_methods();
}
