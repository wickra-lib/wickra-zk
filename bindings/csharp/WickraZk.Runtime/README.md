# Wickra.Zk native runtime packages

`Wickra.Zk.runtime.<rid>` carries the native wickra-zk prover library for one
runtime identifier -- `linux-x64`, `linux-arm64`, `osx-x64` or `osx-arm64` --
under `runtimes/<rid>/native/`. Nothing else: no assembly, no dependencies.

You do not reference these packages yourself. [`Wickra.Zk`](https://www.nuget.org/packages/Wickra.Zk)
depends on all four, so one `dotnet add package Wickra.Zk` brings every
platform along and the host picks the library for the running RID from
`deps.json`, with or without a `RuntimeIdentifier` on the project.

Why four packages rather than one: the prover library is about 90 MB per
platform (the RISC Zero prover and its circuits live inside it), and NuGet.org
caps a package at 250 MB, so the platforms cannot travel together the way the
family's other bindings ship theirs. The version of each runtime package is the
version of `Wickra.Zk` that depends on it.

Source and licence: [wickra-lib/wickra-zk](https://github.com/wickra-lib/wickra-zk),
MIT OR Apache-2.0.
