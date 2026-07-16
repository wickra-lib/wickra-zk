//! Command-line surface.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Prove and verify wickra backtests with zero-knowledge receipts.
#[derive(Parser, Debug)]
#[command(name = "wickra-zk", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Prove an honest backtest over the given data and strategy.
    Prove {
        /// Path to a JSON `ZkSpec` (or a bare `StrategySpec`; the dataset
        /// commitment is computed from the data when absent).
        #[arg(long)]
        spec: PathBuf,
        /// Path to a CSV candle file (`time,open,high,low,close,volume`).
        #[arg(long)]
        data: PathBuf,
        /// Where to write the proof JSON; stdout when omitted.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Request a dev-mode (fast, NOT sound) receipt.
        #[arg(long)]
        dev: bool,
    },
    /// Verify a proof and print its public journal.
    Verify {
        /// Path to a JSON `ZkProof`.
        #[arg(long)]
        proof: PathBuf,
    },
    /// Print the crate version and the pinned guest image id.
    Version,
}
