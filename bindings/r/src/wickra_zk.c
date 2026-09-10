/* R .Call glue for the wickra-zk C ABI hub. */
#include <R.h>
#include <Rinternals.h>
#include <R_ext/Rdynload.h>
#include <stddef.h>
#include "wickra_zk.h"

/* --- handle lifetime ----------------------------------------------------- */

static void wkzk_finalize(SEXP ext) {
    WickraZk *h = (WickraZk *)R_ExternalPtrAddr(ext);
    if (h) {
        wickra_zk_free(h);
    }
    R_ClearExternalPtr(ext);
}

static WickraZk *handle_of(SEXP ext) {
    WickraZk *h = (WickraZk *)R_ExternalPtrAddr(ext);
    if (!h) {
        Rf_error("wickra-zk: handle is closed");
    }
    return h;
}

/* --- exported .Call entries ---------------------------------------------- */

SEXP wkzk_version(void) {
    return Rf_mkString(wickra_zk_version());
}

SEXP wkzk_new(void) {
    WickraZk *h = wickra_zk_new();
    if (!h) {
        Rf_error("wickra-zk: failed to create a prover");
    }
    SEXP ext = PROTECT(R_MakeExternalPtr(h, R_NilValue, R_NilValue));
    R_RegisterCFinalizerEx(ext, wkzk_finalize, TRUE);
    UNPROTECT(1);
    return ext;
}

SEXP wkzk_command(SEXP ext, SEXP cmd_json) {
    WickraZk *h = handle_of(ext);
    const char *cmd = CHAR(STRING_ELT(cmd_json, 0));

    /* Length-out protocol: learn the length, then read into a caller buffer.
       Domain errors come back in-band as {"ok":false,...} JSON, not a negative
       code; only unusable arguments / a caught panic return < 0. */
    int len = wickra_zk_command(h, cmd, NULL, 0);
    if (len < 0) {
        Rf_error("wickra-zk: command failed (code %d)", len);
    }
    char *buf = (char *)R_alloc((size_t)len + 1, 1);
    wickra_zk_command(h, cmd, buf, (size_t)len + 1);
    return Rf_mkString(buf);
}

/* --- registration -------------------------------------------------------- */

static const R_CallMethodDef CallEntries[] = {
    {"wkzk_version", (DL_FUNC)&wkzk_version, 0},
    {"wkzk_new", (DL_FUNC)&wkzk_new, 0},
    {"wkzk_command", (DL_FUNC)&wkzk_command, 2},
    {NULL, NULL, 0}};

void R_init_wickrazk(DllInfo *dll) {
    R_registerRoutines(dll, NULL, CallEntries, NULL, NULL);
    R_useDynamicSymbols(dll, FALSE);
}
