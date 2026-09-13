/* A minimal C example: prove a (strategy, candles) pair through the wickra-zk
 * C ABI, print the public journal, then verify the proof and assert it holds.
 *
 * The prove response is itself a JSON object, so the verify command embeds it
 * verbatim as the "proof" value -- no JSON parser is needed on the C side. The
 * dataset commitment is left to the host: it hashes the candles the same way
 * the guest recomputes them.
 *
 * With RISC0_DEV_MODE=1 the receipt is a fast, unsound placeholder; without it
 * the real prover runs and this takes minutes. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "wickra_zk.h"

static const char *STRATEGY =
    "{\"symbol\":\"AAA\",\"timeframe\":\"1h\","
    "\"indicators\":{\"ema_fast\":{\"type\":\"Ema\",\"params\":[3]},"
    "\"ema_slow\":{\"type\":\"Ema\",\"params\":[8]}},"
    "\"entry\":{\"cross_above\":[\"ema_fast\",\"ema_slow\"]},"
    "\"exit\":{\"cross_below\":[\"ema_fast\",\"ema_slow\"]},"
    "\"sizing\":{\"type\":\"fixed_fraction\",\"fraction\":0.95},"
    "\"costs\":{\"taker_bps\":5,\"slippage\":{\"type\":\"fixed_bps\",\"bps\":2}},"
    "\"risk\":{}}";

/* A short V-shaped price path so the fast/slow EMA cross fires at least once. */
static const char *CANDLES =
    "["
    "{\"time\":1700000000,\"open\":120,\"high\":121,\"low\":119,\"close\":120,\"volume\":1000},"
    "{\"time\":1700003600,\"open\":120,\"high\":121,\"low\":117,\"close\":118,\"volume\":1000},"
    "{\"time\":1700007200,\"open\":118,\"high\":119,\"low\":115,\"close\":116,\"volume\":1000},"
    "{\"time\":1700010800,\"open\":116,\"high\":117,\"low\":113,\"close\":114,\"volume\":1000},"
    "{\"time\":1700014400,\"open\":114,\"high\":115,\"low\":111,\"close\":112,\"volume\":1000},"
    "{\"time\":1700018000,\"open\":112,\"high\":113,\"low\":109,\"close\":110,\"volume\":1000},"
    "{\"time\":1700021600,\"open\":110,\"high\":111,\"low\":107,\"close\":108,\"volume\":1000},"
    "{\"time\":1700025200,\"open\":108,\"high\":113,\"low\":107,\"close\":112,\"volume\":1000},"
    "{\"time\":1700028800,\"open\":112,\"high\":117,\"low\":111,\"close\":116,\"volume\":1000},"
    "{\"time\":1700032400,\"open\":116,\"high\":121,\"low\":115,\"close\":120,\"volume\":1000},"
    "{\"time\":1700036000,\"open\":120,\"high\":125,\"low\":119,\"close\":124,\"volume\":1000},"
    "{\"time\":1700039600,\"open\":124,\"high\":129,\"low\":123,\"close\":128,\"volume\":1000}]";

/* Read a command response into a freshly malloc'd, NUL-terminated buffer using
 * the length-out protocol. Returns NULL on failure. */
static char *run(WickraZk *prover, const char *cmd) {
    int32_t len = wickra_zk_command(prover, cmd, NULL, 0);
    if (len < 0) {
        fprintf(stderr, "command failed: code %d\n", (int)len);
        return NULL;
    }
    char *buf = (char *)malloc((size_t)len + 1);
    if (!buf) {
        return NULL;
    }
    wickra_zk_command(prover, cmd, buf, (size_t)len + 1);
    return buf;
}

int main(void) {
    WickraZk *prover = wickra_zk_new();
    if (!prover) {
        fprintf(stderr, "failed to create prover\n");
        return 1;
    }

    size_t prove_cap = strlen(STRATEGY) + strlen(CANDLES) + 64;
    char *prove_cmd = (char *)malloc(prove_cap);
    if (!prove_cmd) {
        wickra_zk_free(prover);
        return 1;
    }
    snprintf(prove_cmd, prove_cap, "{\"cmd\":\"prove\",\"spec\":{\"strategy\":%s},\"candles\":%s}", STRATEGY,
             CANDLES);

    char *proof = run(prover, prove_cmd);
    if (!proof) {
        free(prove_cmd);
        wickra_zk_free(prover);
        return 1;
    }

    printf("wickra-zk %s\n", wickra_zk_version());
    /* The receipt carries its own "journal" (the committed bytes); the public
     * outputs are the object that opens with report_hash. */
    const char *journal = strstr(proof, "\"journal\":{\"report_hash\":");
    if (!journal) {
        fprintf(stderr, "prove did not return a journal: %s\n", proof);
        free(proof);
        free(prove_cmd);
        wickra_zk_free(prover);
        return 1;
    }
    const char *journal_end = strchr(journal, '}');
    printf("proof: %.*s}\n", (int)(journal_end - journal), journal);

    /* Verify: the prove response is valid JSON, so it drops straight in as the
     * "proof" value. The outputs come back decoded from the receipt. */
    size_t verify_cap = strlen(proof) + 64;
    char *verify_cmd = (char *)malloc(verify_cap);
    if (!verify_cmd) {
        free(proof);
        free(prove_cmd);
        wickra_zk_free(prover);
        return 1;
    }
    snprintf(verify_cmd, verify_cap, "{\"cmd\":\"verify\",\"proof\":%s}", proof);

    char *outputs = run(prover, verify_cmd);
    int ok = outputs && strncmp(outputs, journal + strlen("\"journal\":"), (size_t)(journal_end - journal) - strlen("\"journal\":") + 1) == 0;
    printf("verify: %s\n", ok ? "valid" : "INVALID");

    free(outputs);
    free(verify_cmd);
    free(proof);
    free(prove_cmd);
    wickra_zk_free(prover);

    if (!ok) {
        fprintf(stderr, "verification did not hold\n");
        return 1;
    }
    return 0;
}
