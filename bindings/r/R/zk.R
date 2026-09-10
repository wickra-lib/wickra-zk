#' The wickra-zk library version.
#' @return A version string.
#' @export
wkzk_version <- function() {
  .Call(C_wkzk_version)
}

#' Create a stateless prover.
#' @return A `wickra_zk` handle (an external pointer).
#' @export
wkzk_new <- function() {
  .Call(C_wkzk_new)
}

#' Apply a command JSON and return the resulting response JSON.
#' @param prover A prover handle from [wkzk_new()].
#' @param cmd_json A command JSON string.
#' @return The response as a JSON string.
#' @export
wkzk_command <- function(prover, cmd_json) {
  .Call(C_wkzk_command, prover, cmd_json)
}
