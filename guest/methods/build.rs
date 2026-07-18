//! Build the detached RISC-V guest crate and embed its ELF + image ID.
//!
//! `risc0_build::embed_methods()` compiles `guest/` for the
//! `riscv32im-risc0-zkvm-elf` target and writes `$OUT_DIR/methods.rs`, which
//! `src/lib.rs` includes. The generated constants are `WICKRA_ZK_GUEST_ELF`
//! (the program bytes handed to the prover) and `WICKRA_ZK_GUEST_ID` (the
//! image ID the verifier pins).

fn main() {
    risc0_build::embed_methods();
}
