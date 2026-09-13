# Prove a backtest in zero knowledge from R, then verify the proof.
#
# Reads the shared example inputs (../specs/momentum.json and
# ../data/BTCUSDT.csv, relative to this script), proves them through the
# command envelope -- the dataset commitment is left to the host -- verifies
# the proof and prints the public journal. With RISC0_DEV_MODE=1 the receipt
# is a fast, unsound placeholder; without it the real prover runs and this
# takes minutes.
#
#   cargo build -p wickra-zk-c --release
#   export WKZK_INC="$PWD/bindings/c/include"
#   export WKZK_LIB="$PWD/target/release"
#   export LD_LIBRARY_PATH="$WKZK_LIB:$LD_LIBRARY_PATH"   # DYLD_LIBRARY_PATH on macOS
#   R CMD INSTALL bindings/r
#   RISC0_DEV_MODE=1 Rscript examples/r/prove.R

library(wickrazk)

script_dir <- function() {
  args <- commandArgs(trailingOnly = FALSE)
  file <- sub("^--file=", "", args[grepl("^--file=", args)])
  if (length(file) == 1) dirname(normalizePath(file)) else getwd()
}
examples <- file.path(script_dir(), "..")

read_text <- function(path) trimws(paste(readLines(path, warn = FALSE), collapse = "\n"))

load_candles <- function(path) {
  rows <- character(0)
  for (line in trimws(readLines(path, warn = FALSE))) {
    cols <- trimws(strsplit(line, ",")[[1]])
    if (length(cols) < 6 || !grepl("^[0-9]+$", cols[1])) next # header
    rows <- c(rows, paste0(
      '{"time":', cols[1], ',"open":', cols[2], ',"high":', cols[3],
      ',"low":', cols[4], ',"close":', cols[5], ',"volume":', cols[6], "}"
    ))
  }
  paste0("[", paste(rows, collapse = ","), "]")
}

field <- function(json, key) {
  m <- regmatches(json, regexpr(paste0('"', key, '":"[0-9a-f]{64}"'), json))
  stopifnot(length(m) == 1)
  sub(paste0('"', key, '":"([0-9a-f]{64})"'), "\\1", m)
}

strategy <- read_text(file.path(examples, "specs", "momentum.json"))
candles <- load_candles(file.path(examples, "data", "BTCUSDT.csv"))

prover <- wkzk_new()
proof <- wkzk_command(prover, paste0(
  '{"cmd":"prove","spec":{"strategy":', strategy, '},"candles":', candles, "}"
))
stopifnot(!startsWith(proof, '{"ok":false'))

# The receipt carries its own "journal" (the committed bytes); the public
# outputs are the object that opens with report_hash.
span <- regexpr('"journal":\\{"report_hash":[^{}]*\\}', proof, perl = TRUE)
stopifnot(span > 0)
persisted <- sub('^"journal":', "", regmatches(proof, span), perl = TRUE)

journal <- wkzk_command(prover, paste0('{"cmd":"verify","proof":', proof, "}"))

cat("wickra-zk", wkzk_version(), "\n")
cat("guest_id:", field(journal, "guest_id"), "\n")
cat("report_hash:", field(journal, "report_hash"), "\n")
cat(if (identical(journal, persisted)) "verify: valid\n" else "verify: INVALID\n")
cat(journal, "\n")
