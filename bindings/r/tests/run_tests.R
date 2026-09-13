## Plain-R tests for the wickra-zk R binding (no testthat dependency).
##
## Cross-language golden parity through the command envelope, in dev-mode.
## Each golden case is proven through the binding without a stated commitment,
## and the journal must carry the blessed report_hash and metrics -- the same
## determinism chain crates/wickra-zk-host/tests/golden.rs pins natively. The
## proof then verifies to the same journal, a proof file whose journal disagrees
## with its receipt is refused, and a stated commitment that is not the hash of
## the candles is refused before the zkVM runs.
##
## Dev-mode receipts are unsound and fast; this tests the binding's transport
## of the envelope, not the proof system. The real prover runs nightly. Base R
## has no JSON reader, so fields are read back by pattern; the journal is flat,
## so its object is one brace pair.

library(wickrazk)

## The prover reads this through getenv; Sys.setenv reaches it.
Sys.setenv(RISC0_DEV_MODE = "1")

golden_dir <- function() {
  d <- normalizePath(getwd(), mustWork = FALSE)
  for (i in seq_len(10)) {
    candidate <- file.path(d, "golden")
    if (file.exists(file.path(candidate, "cases.json"))) {
      return(candidate)
    }
    d <- dirname(d)
  }
  stop("golden/cases.json not found above the working directory")
}

read_text <- function(path) trimws(paste(readLines(path, warn = FALSE), collapse = "\n"))

string_field <- function(json, key) {
  m <- regmatches(json, regexpr(paste0('"', key, '"\\s*:\\s*"[^"]*"'), json, perl = TRUE))
  stopifnot(length(m) == 1)
  sub(paste0('"', key, '"\\s*:\\s*"([^"]*)"'), "\\1", m, perl = TRUE)
}

number_field <- function(json, key) {
  m <- regmatches(json, regexpr(paste0('"', key, '"\\s*:\\s*-?[0-9.eE+-]+'), json, perl = TRUE))
  stopifnot(length(m) == 1)
  as.numeric(sub(paste0('"', key, '"\\s*:\\s*'), "", m, perl = TRUE))
}

in_band_error <- function(response) {
  if (grepl('^\\{"ok":false,"error":"', response, perl = TRUE)) {
    sub('^\\{"ok":false,"error":"(.*)"\\}$', "\\1", response, perl = TRUE)
  } else {
    NULL
  }
}

load_cases <- function(g) {
  text <- read_text(file.path(g, "cases.json"))
  pairs <- regmatches(text, gregexpr('"[^"]+"\\s*:\\s*"[^"]+"', text, perl = TRUE))[[1]]
  stopifnot(length(pairs) > 0)
  keys <- sub('^"([^"]+)".*$', "\\1", pairs, perl = TRUE)
  values <- sub('^.*:\\s*"([^"]+)"$', "\\1", pairs, perl = TRUE)
  setNames(values, keys)[order(keys)]
}

load_candles <- function(g, dataset) {
  rows <- character(0)
  for (line in trimws(readLines(file.path(g, "data", paste0(dataset, ".csv")), warn = FALSE))) {
    cols <- trimws(strsplit(line, ",")[[1]])
    if (length(cols) < 6 || !grepl("^[0-9]+$", cols[1])) next # header
    rows <- c(rows, paste0(
      '{"time":', cols[1], ',"open":', cols[2], ',"high":', cols[3],
      ',"low":', cols[4], ',"close":', cols[5], ',"volume":', cols[6], "}"
    ))
  }
  paste0("[", paste(rows, collapse = ","), "]")
}

## The receipt carries its own "journal" (the committed bytes); the persisted
## public outputs are the object that opens with report_hash.
journal_of <- function(proof) {
  m <- regexpr('"journal":\\{"report_hash":[^{}]*\\}', proof, perl = TRUE)
  stopifnot(m > 0)
  list(
    text = sub('^"journal":', "", regmatches(proof, m), perl = TRUE),
    start = m,
    end = m + attr(m, "match.length") - 1L
  )
}

g <- golden_dir()
cases <- load_cases(g)
prover <- wkzk_new()

## version is reported two ways and they agree
stopifnot(nzchar(wkzk_version()))
version <- wkzk_command(prover, '{"cmd":"version"}')
stopifnot(identical(string_field(version, "version"), wkzk_version()))
guest_id <- string_field(version, "guest_id")

## a malformed envelope is an in-band error
stopifnot(!is.null(in_band_error(wkzk_command(prover, "not json"))))

## golden cases prove to the blessed journal
for (name in names(cases)) {
  strategy <- read_text(file.path(g, "specs", paste0(name, ".json")))
  expected <- read_text(file.path(g, "expected", paste0(name, ".json")))
  candles <- load_candles(g, cases[[name]])

  commit <- wkzk_command(prover, paste0('{"cmd":"commit","candles":', candles, "}"))
  stopifnot(is.null(in_band_error(commit)))
  proof <- wkzk_command(prover, paste0(
    '{"cmd":"prove","spec":{"strategy":', strategy, '},"candles":', candles, "}"
  ))
  stopifnot(is.null(in_band_error(proof)))
  span <- journal_of(proof)
  journal <- span$text

  stopifnot(identical(string_field(journal, "report_hash"), string_field(expected, "report_hash")))
  stopifnot(number_field(journal, "n_trades") == number_field(expected, "n_trades"))
  stopifnot(abs(number_field(journal, "sharpe") - number_field(expected, "sharpe")) < 1e-8)
  stopifnot(abs(number_field(journal, "pnl") - number_field(expected, "pnl")) < 1e-8)
  stopifnot(identical(string_field(journal, "dataset_commitment"), string_field(commit, "dataset_commitment")))
  stopifnot(identical(string_field(journal, "guest_id"), guest_id))

  outputs <- wkzk_command(prover, paste0('{"cmd":"verify","proof":', proof, "}"))
  stopifnot(identical(outputs, journal))

  lying <- paste0(
    substr(proof, 1, span$start - 1L),
    '"journal":',
    sub('"report_hash":"[0-9a-f]{64}"', paste0('"report_hash":"', strrep("f", 64), '"'), journal, perl = TRUE),
    substr(proof, span$end + 1L, nchar(proof))
  )
  refused <- in_band_error(wkzk_command(prover, paste0('{"cmd":"verify","proof":', lying, "}")))
  stopifnot(!is.null(refused), grepl("verify", refused, fixed = TRUE))
}

## a stated commitment is held to
first <- names(cases)[1]
refused <- in_band_error(wkzk_command(prover, paste0(
  '{"cmd":"prove","spec":{"strategy":', read_text(file.path(g, "specs", paste0(first, ".json"))),
  ',"dataset_commitment":"', strrep("0", 64), '"},"candles":', load_candles(g, cases[[first]]), "}"
)))
stopifnot(!is.null(refused), grepl("commitment mismatch", refused, fixed = TRUE))

cat("wickra-zk R tests passed:", length(cases), "golden cases\n")
