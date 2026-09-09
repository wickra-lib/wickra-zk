<!-- Keep it short. One logical change per PR.

     Changing the release process, the bindings' shared surface, or the public
     API? There is a longer template that asks what such a change has to answer:
     reopen this pull request with ?template=detailed.md appended to the URL.
     GitHub offers no picker for a second template, so it is only reachable that
     way. -->

## What

<!-- What does this change and why? -->

## Checklist

- [ ] `cargo fmt --all` and `cargo clippy --workspace --all-targets --all-features -- -D warnings` are clean
- [ ] Guest clippy is clean (`cd guest/methods/guest && cargo clippy --target riscv32im-risc0-zkvm-elf -- -D warnings`)
- [ ] `RISC0_DEV_MODE=1 cargo test --workspace --all-features` passes (dev-mode prove/verify roundtrip green)
- [ ] Golden hashes match (guest journal `report_hash` == native `wickra-proof` hash)
- [ ] `cargo deny check` is clean
- [ ] Determinism preserved (no HashMap/time/RNG/threads in the guest report path)
- [ ] `CHANGELOG.md` updated under `[Unreleased]`
