//! `wickra-zk` — command-line prover/verifier.

mod args;
mod run;

use std::process::ExitCode;

use clap::Parser;

use crate::args::Cli;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run::run(cli.command) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
