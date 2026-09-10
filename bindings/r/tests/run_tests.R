## Plain-R tests for the wickra-zk R binding (no testthat dependency).
## Mirrors the Rust/Python/Node/Go/C#/Java tests and doubles as the completeness
## guard: it exercises the full public surface (version + new + command).

library(wickrazk)

strategy <- paste0(
  '{"symbol":"BTCUSDT","timeframe":"1h",',
  '"indicators":{"ema_fast":{"type":"Ema","params":[5]},',
  '"ema_slow":{"type":"Ema","params":[15]}},',
  '"entry":{"cross_above":["ema_fast","ema_slow"]},',
  '"exit":{"cross_below":["ema_fast","ema_slow"]},',
  '"sizing":{"type":"fixed_fraction","fraction":0.95},',
  '"costs":{"taker_bps":5,"slippage":{"type":"fixed_bps","bps":2}},',
  '"risk":{"trailing_stop_pct":5.0}}'
)

candles <- function() {
  parts <- vapply(0:39, function(i) {
    b <- 100.0 + sin(i * 0.4) * 8.0
    paste0(
      '{"time":', format(1700000000 + i * 3600, scientific = FALSE),
      ',"open":', b, ',"high":', b + 1.0, ',"low":', b - 1.0,
      ',"close":', b + 0.5, ',"volume":1000.0}'
    )
  }, character(1))
  paste0("[", paste(parts, collapse = ","), "]")
}

spec <- paste0('{"strategy":', strategy, ',"dataset_ref":"BTCUSDT/1h/test"}')
data <- paste0('{"BTCUSDT":', candles(), '}')

prove <- function(prover) {
  wkzk_command(prover, paste0('{"cmd":"prove","spec":', spec, ',"data":', data, '}'))
}

hex_field <- function(json, key) {
  m <- regmatches(json, regexpr(paste0('"', key, '":"[0-9a-f]{64}"'), json))
  stopifnot(length(m) == 1)
  m
}

## version
stopifnot(nzchar(wkzk_version()))

## prove -> 64-hex report_hash + inputs_hash
prover <- wkzk_new()
proof <- prove(prover)
stopifnot(nchar(hex_field(proof, "report_hash")) == 64 + nchar('"report_hash":""'))
stopifnot(nchar(hex_field(proof, "inputs_hash")) == 64 + nchar('"inputs_hash":""'))

## prove is reproducible
stopifnot(identical(
  hex_field(prove(wkzk_new()), "report_hash"),
  hex_field(prove(wkzk_new()), "report_hash")
))

## verify accepts a genuine proof and rejects a tampered one
good <- wkzk_command(
  prover,
  paste0('{"cmd":"verify","proof":', proof, ',"spec":', spec, ',"data":', data, '}')
)
stopifnot(identical(good, '{"ok":true,"valid":true}'))

tampered <- sub(
  '"report_hash":"[0-9a-f]{64}"',
  paste0('"report_hash":"', strrep("0", 64), '"'),
  proof
)
bad <- wkzk_command(
  prover,
  paste0('{"cmd":"verify","proof":', tampered, ',"spec":', spec, ',"data":', data, '}')
)
stopifnot(identical(bad, '{"ok":true,"valid":false}'))

## an unknown command is an in-band error, not a hard error
inband <- wkzk_command(prover, '{"cmd":"nope"}')
stopifnot(grepl('"ok":false', inband, fixed = TRUE))

## cross-language golden parity: for each committed golden/specs/*.json, prove
## over the shared golden/data.json and assert the response equals
## golden/expected/<spec>.json byte-for-byte. The binding returns the core's
## canonical command output verbatim, so byte equality is the exact
## cross-language parity check. The fixtures arrive in a later phase; until then
## the golden section is skipped.
golden_dir <- function() {
  d <- normalizePath(getwd(), mustWork = FALSE)
  for (i in seq_len(8)) {
    g <- file.path(d, "golden")
    if (dir.exists(file.path(g, "specs"))) {
      return(g)
    }
    d <- dirname(d)
  }
  NULL
}

g <- golden_dir()
if (!is.null(g)) {
  dataset <- trimws(paste(
    readLines(file.path(g, "data.json"), warn = FALSE), collapse = "\n"
  ))
  for (spec_path in list.files(file.path(g, "specs"), pattern = "\\.json$", full.names = TRUE)) {
    name <- basename(spec_path)
    spec_json <- trimws(paste(readLines(spec_path, warn = FALSE), collapse = "\n"))
    expected <- trimws(paste(
      readLines(file.path(g, "expected", name), warn = FALSE), collapse = "\n"
    ))
    gprover <- wkzk_new()
    got <- wkzk_command(
      gprover, paste0('{"cmd":"prove","spec":', spec_json, ',"data":', dataset, '}')
    )
    stopifnot(identical(trimws(got), expected))
  }
}

cat("wickra-zk R tests passed\n")
