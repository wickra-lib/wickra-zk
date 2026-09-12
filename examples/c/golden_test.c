/* Cross-language golden parity through the command envelope, from C.
 *
 * Each golden case is proven through the C ABI without a stated commitment,
 * and the journal must carry the blessed report_hash and metrics -- the same
 * determinism chain crates/wickra-zk-host/tests/golden.rs pins natively. The
 * proof then verifies to the same journal, a proof file whose journal disagrees
 * with its receipt is refused, and a stated commitment that is not the hash of
 * the candles is refused before the zkVM runs.
 *
 * Until this existed the C ABI was the only reach with no test at all beyond
 * the version command. Four of the seven language reaches go through this ABI,
 * so a fault here is a fault in all of them.
 *
 * ctest sets RISC0_DEV_MODE=1: a dev-mode receipt takes milliseconds, a real
 * one takes minutes, and what is exercised here is the envelope, not the proof
 * system. The real prover runs nightly.
 *
 * C has no directory API that is portable, so the case list is globbed by
 * CMake at configure time into golden_cases.h; the case -> dataset mapping is
 * read from golden/cases.json, the file every other binding's test reads. A
 * case added to the corpus is covered here without editing this file.
 */
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "wickra_zk.h"

#include "golden_cases.h" /* GOLDEN_DIR, GOLDEN_CASES, GOLDEN_CASE_COUNT */

/* Read a whole file. Caller frees. Returns NULL and reports on failure. */
static char *slurp(const char *path) {
    FILE *file = fopen(path, "rb");
    if (!file) {
        fprintf(stderr, "cannot open %s\n", path);
        return NULL;
    }
    if (fseek(file, 0, SEEK_END) != 0) {
        fclose(file);
        return NULL;
    }
    long size = ftell(file);
    if (size < 0 || fseek(file, 0, SEEK_SET) != 0) {
        fclose(file);
        return NULL;
    }
    char *buf = (char *)malloc((size_t)size + 1);
    if (!buf) {
        fclose(file);
        return NULL;
    }
    size_t got = fread(buf, 1, (size_t)size, file);
    fclose(file);
    buf[got] = '\0';
    return buf;
}

/* A growable string. */
typedef struct {
    char *buf;
    size_t len;
    size_t cap;
} Str;

static int str_push(Str *s, const char *text, size_t n) {
    if (s->len + n + 1 > s->cap) {
        size_t cap = s->cap ? s->cap : 4096;
        while (cap < s->len + n + 1) {
            cap *= 2;
        }
        char *grown = (char *)realloc(s->buf, cap);
        if (!grown) {
            return 0;
        }
        s->buf = grown;
        s->cap = cap;
    }
    memcpy(s->buf + s->len, text, n);
    s->len += n;
    s->buf[s->len] = '\0';
    return 1;
}

static int str_puts(Str *s, const char *text) { return str_push(s, text, strlen(text)); }

/* The dataset name a case maps to in cases.json, into `out`. */
static int dataset_of(const char *cases_json, const char *name, char *out, size_t cap) {
    char key[128];
    snprintf(key, sizeof key, "\"%s\"", name);
    const char *at = strstr(cases_json, key);
    if (!at) {
        return 0;
    }
    at = strchr(at + strlen(key), ':');
    if (!at) {
        return 0;
    }
    at = strchr(at, '"');
    if (!at) {
        return 0;
    }
    const char *end = strchr(at + 1, '"');
    if (!end || (size_t)(end - at) >= cap) {
        return 0;
    }
    memcpy(out, at + 1, (size_t)(end - at - 1));
    out[end - at - 1] = '\0';
    return 1;
}

/* golden/data/<dataset>.csv as a JSON candle array. Column text is passed
 * through verbatim, as the other bindings do. */
static int load_candles(Str *out, const char *dataset) {
    char path[1024];
    snprintf(path, sizeof path, "%s/data/%s.csv", GOLDEN_DIR, dataset);
    char *raw = slurp(path);
    if (!raw || !str_puts(out, "[")) {
        free(raw);
        return 0;
    }
    static const char *KEYS[] = {"time", "open", "high", "low", "close", "volume"};
    int first = 1;
    char *line = raw;
    while (line && *line) {
        char *next = strpbrk(line, "\r\n");
        if (next) {
            *next++ = '\0';
            while (*next == '\r' || *next == '\n') {
                next++;
            }
        }
        char *cols[6];
        int n = 0;
        char *col = line;
        while (col && n < 6) {
            char *comma = strchr(col, ',');
            if (comma) {
                *comma++ = '\0';
            }
            while (*col == ' ') {
                col++;
            }
            cols[n++] = col;
            col = comma;
        }
        if (n == 6 && cols[0][0] >= '0' && cols[0][0] <= '9') {
            if (!first) {
                str_puts(out, ",");
            }
            first = 0;
            str_puts(out, "{");
            for (int k = 0; k < 6; k++) {
                str_puts(out, k ? ",\"" : "\"");
                str_puts(out, KEYS[k]);
                str_puts(out, "\":");
                str_puts(out, cols[k]);
            }
            str_puts(out, "}");
        }
        line = next;
    }
    free(raw);
    return str_puts(out, "]");
}

/* Apply one command through the two-call length protocol. Caller frees. */
static char *run(WickraZk *prover, const char *cmd) {
    int32_t len = wickra_zk_command(prover, cmd, NULL, 0);
    if (len < 0) {
        fprintf(stderr, "command failed with code %d\n", (int)len);
        return NULL;
    }
    char *out = (char *)malloc((size_t)len + 1);
    if (!out) {
        return NULL;
    }
    if (wickra_zk_command(prover, cmd, out, (size_t)len + 1) < 0) {
        free(out);
        return NULL;
    }
    return out;
}

/* The in-band error message of a response, or NULL when it is not an error. */
static const char *in_band_error(const char *response) {
    static const char PREFIX[] = "{\"ok\":false,\"error\":\"";
    return strncmp(response, PREFIX, sizeof PREFIX - 1) == 0 ? response + sizeof PREFIX - 1 : NULL;
}

/* The value of `"key":"..."` (whitespace around the colon allowed), into `out`. */
static int string_field(const char *json, const char *key, char *out, size_t cap) {
    char quoted[128];
    snprintf(quoted, sizeof quoted, "\"%s\"", key);
    const char *at = strstr(json, quoted);
    if (!at) {
        return 0;
    }
    at += strlen(quoted);
    while (*at == ' ' || *at == ':' || *at == '\n') {
        at++;
    }
    if (*at != '"') {
        return 0;
    }
    const char *end = strchr(at + 1, '"');
    if (!end || (size_t)(end - at) >= cap) {
        return 0;
    }
    memcpy(out, at + 1, (size_t)(end - at - 1));
    out[end - at - 1] = '\0';
    return 1;
}

/* The value of `"key":<number>` (whitespace around the colon allowed). */
static int number_field(const char *json, const char *key, double *out) {
    char quoted[128];
    snprintf(quoted, sizeof quoted, "\"%s\"", key);
    const char *at = strstr(json, quoted);
    if (!at) {
        return 0;
    }
    at += strlen(quoted);
    while (*at == ' ' || *at == ':' || *at == '\n') {
        at++;
    }
    char *stop = NULL;
    *out = strtod(at, &stop);
    return stop != at;
}

int main(void) {
    if (GOLDEN_CASE_COUNT == 0) {
        fprintf(stderr, "no golden cases were configured; this would test nothing\n");
        return 1;
    }
    if (!getenv("RISC0_DEV_MODE") || strcmp(getenv("RISC0_DEV_MODE"), "1") != 0) {
        fprintf(stderr, "set RISC0_DEV_MODE=1: this test drives the envelope, not the real prover\n");
        return 1;
    }

    char path[1024];
    snprintf(path, sizeof path, "%s/cases.json", GOLDEN_DIR);
    char *cases_json = slurp(path);
    if (!cases_json) {
        return 1;
    }

    WickraZk *prover = wickra_zk_new();
    if (!prover) {
        fprintf(stderr, "failed to create prover\n");
        return 1;
    }
    char *version = run(prover, "{\"cmd\":\"version\"}");
    char guest_id[80];
    if (!version || !string_field(version, "guest_id", guest_id, sizeof guest_id)) {
        fprintf(stderr, "version did not name the guest\n");
        return 1;
    }
    free(version);

    int failures = 0;
    for (size_t i = 0; i < GOLDEN_CASE_COUNT; i++) {
        const char *name = GOLDEN_CASES[i];
        char dataset[128];
        if (!dataset_of(cases_json, name, dataset, sizeof dataset)) {
            fprintf(stderr, "%s: not in cases.json\n", name);
            failures++;
            continue;
        }
        snprintf(path, sizeof path, "%s/specs/%s.json", GOLDEN_DIR, name);
        char *strategy = slurp(path);
        snprintf(path, sizeof path, "%s/expected/%s.json", GOLDEN_DIR, name);
        char *expected = slurp(path);
        Str candles = {0};
        if (!strategy || !expected || !load_candles(&candles, dataset)) {
            fprintf(stderr, "%s: could not load the case\n", name);
            failures++;
            free(strategy);
            free(expected);
            free(candles.buf);
            continue;
        }

        Str cmd = {0};
        str_puts(&cmd, "{\"cmd\":\"commit\",\"candles\":");
        str_puts(&cmd, candles.buf);
        str_puts(&cmd, "}");
        char *commit = run(prover, cmd.buf);
        cmd.len = 0;
        str_puts(&cmd, "{\"cmd\":\"prove\",\"spec\":{\"strategy\":");
        str_puts(&cmd, strategy);
        str_puts(&cmd, "},\"candles\":");
        str_puts(&cmd, candles.buf);
        str_puts(&cmd, "}");
        char *proof = run(prover, cmd.buf);
        free(cmd.buf);

        /* The receipt carries its own "journal" (the committed bytes); the
         * persisted public outputs are the object that opens with report_hash. */
        const char *journal = proof ? strstr(proof, "\"journal\":{\"report_hash\":") : NULL;
        if (!commit || in_band_error(commit) || !proof || in_band_error(proof) || !journal) {
            fprintf(stderr, "%s: prove failed: %s\n", name, proof ? proof : "(no response)");
            failures++;
        } else {
            journal += strlen("\"journal\":");
            size_t journal_len = (size_t)(strchr(journal, '}') - journal) + 1;

            char got[80], want[80];
            double got_n, want_n;
            if (!string_field(journal, "report_hash", got, sizeof got) ||
                !string_field(expected, "report_hash", want, sizeof want) || strcmp(got, want) != 0) {
                fprintf(stderr, "%s: report_hash %s, blessed %s\n", name, got, want);
                failures++;
            }
            if (!number_field(journal, "n_trades", &got_n) || !number_field(expected, "n_trades", &want_n) ||
                got_n != want_n) {
                fprintf(stderr, "%s: n_trades drift\n", name);
                failures++;
            }
            if (!number_field(journal, "sharpe", &got_n) || !number_field(expected, "sharpe", &want_n) ||
                fabs(got_n - want_n) >= 1e-8) {
                fprintf(stderr, "%s: sharpe drift\n", name);
                failures++;
            }
            if (!number_field(journal, "pnl", &got_n) || !number_field(expected, "pnl", &want_n) ||
                fabs(got_n - want_n) >= 1e-8) {
                fprintf(stderr, "%s: pnl drift\n", name);
                failures++;
            }
            if (!string_field(journal, "dataset_commitment", got, sizeof got) ||
                !string_field(commit, "dataset_commitment", want, sizeof want) || strcmp(got, want) != 0) {
                fprintf(stderr, "%s: the journal is not bound to the candles\n", name);
                failures++;
            }
            if (!string_field(journal, "guest_id", got, sizeof got) || strcmp(got, guest_id) != 0) {
                fprintf(stderr, "%s: guest_id %s, host pins %s\n", name, got, guest_id);
                failures++;
            }

            Str verify_cmd = {0};
            str_puts(&verify_cmd, "{\"cmd\":\"verify\",\"proof\":");
            str_puts(&verify_cmd, proof);
            str_puts(&verify_cmd, "}");
            char *outputs = run(prover, verify_cmd.buf);
            if (!outputs || strlen(outputs) != journal_len || strncmp(outputs, journal, journal_len) != 0) {
                fprintf(stderr, "%s: verify returned %s\n  journal is %.*s\n", name, outputs ? outputs : "(none)",
                        (int)journal_len, journal);
                failures++;
            }
            free(outputs);

            /* The receipt is untouched; only the human-readable copy beside it lies. */
            static const char VERIFY_PREFIX[] = "{\"cmd\":\"verify\",\"proof\":";
            char *hash_at =
                strstr(verify_cmd.buf + sizeof VERIFY_PREFIX - 1 + (journal - proof), "\"report_hash\":\"");
            memset(hash_at + strlen("\"report_hash\":\""), 'f', 64);
            char *refused = run(prover, verify_cmd.buf);
            const char *message = refused ? in_band_error(refused) : NULL;
            if (!message || !strstr(message, "verify")) {
                fprintf(stderr, "%s: a proof file whose journal lies must be refused, got %s\n", name,
                        refused ? refused : "(none)");
                failures++;
            }
            free(refused);
            free(verify_cmd.buf);
        }

        /* A stated commitment is held to. */
        if (i == 0) {
            Str wrong = {0};
            str_puts(&wrong, "{\"cmd\":\"prove\",\"spec\":{\"strategy\":");
            str_puts(&wrong, strategy);
            str_puts(&wrong, "},\"candles\":");
            str_puts(&wrong, candles.buf);
            str_puts(&wrong, "}");
            /* Insert the false commitment before the strategy's closing brace. */
            const char *tail = "},\"candles\":";
            char *at = strstr(wrong.buf, tail);
            Str with = {0};
            str_push(&with, wrong.buf, (size_t)(at - wrong.buf));
            str_puts(&with, ",\"dataset_commitment\":\"");
            for (int k = 0; k < 64; k++) {
                str_puts(&with, "0");
            }
            str_puts(&with, "\"");
            str_puts(&with, at);
            char *refused = run(prover, with.buf);
            const char *message = refused ? in_band_error(refused) : NULL;
            if (!message || !strstr(message, "commitment mismatch")) {
                fprintf(stderr, "a false commitment must be refused, got %s\n", refused ? refused : "(none)");
                failures++;
            }
            free(refused);
            free(with.buf);
            free(wrong.buf);
        }

        free(commit);
        free(proof);
        free(candles.buf);
        free(strategy);
        free(expected);
    }

    wickra_zk_free(prover);
    free(cases_json);

    if (failures > 0) {
        fprintf(stderr, "%d failure(s) across %zu golden cases\n", failures, GOLDEN_CASE_COUNT);
        return 1;
    }
    printf("all %zu golden cases prove to the blessed journal from C\n", GOLDEN_CASE_COUNT);
    return 0;
}
