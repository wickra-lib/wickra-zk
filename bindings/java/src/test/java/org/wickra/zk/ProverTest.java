package org.wickra.zk;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.regex.Matcher;
import java.util.regex.Pattern;
import org.junit.jupiter.api.Test;

class ProverTest {
    private static final String STRATEGY =
            "{\"symbol\":\"BTCUSDT\",\"timeframe\":\"1h\","
                    + "\"indicators\":{\"ema_fast\":{\"type\":\"Ema\",\"params\":[5]},"
                    + "\"ema_slow\":{\"type\":\"Ema\",\"params\":[15]}},"
                    + "\"entry\":{\"cross_above\":[\"ema_fast\",\"ema_slow\"]},"
                    + "\"exit\":{\"cross_below\":[\"ema_fast\",\"ema_slow\"]},"
                    + "\"sizing\":{\"type\":\"fixed_fraction\",\"fraction\":0.95},"
                    + "\"costs\":{\"taker_bps\":5,\"slippage\":{\"type\":\"fixed_bps\",\"bps\":2}},"
                    + "\"risk\":{\"trailing_stop_pct\":5.0}}";

    private static String candles() {
        StringBuilder sb = new StringBuilder("[");
        for (int i = 0; i < 40; i++) {
            double b = 100.0 + Math.sin(i * 0.4) * 8.0;
            if (i > 0) {
                sb.append(',');
            }
            sb.append("{\"time\":").append(1_700_000_000L + i * 3600L)
                    .append(",\"open\":").append(b)
                    .append(",\"high\":").append(b + 1.0)
                    .append(",\"low\":").append(b - 1.0)
                    .append(",\"close\":").append(b + 0.5)
                    .append(",\"volume\":1000.0}");
        }
        return sb.append(']').toString();
    }

    private static final String SPEC =
            "{\"strategy\":" + STRATEGY + ",\"dataset_ref\":\"BTCUSDT/1h/test\"}";
    private static final String DATA = "{\"BTCUSDT\":" + candles() + "}";

    private static String prove(Prover prover) {
        return prover.command("{\"cmd\":\"prove\",\"spec\":" + SPEC + ",\"data\":" + DATA + "}");
    }

    private static String field(String json, String key) {
        Matcher m = Pattern.compile("\"" + key + "\":\"([0-9a-f]{64})\"").matcher(json);
        assertTrue(m.find(), "missing " + key + " in " + json);
        return m.group(1);
    }

    @Test
    void versionIsNonEmpty() {
        assertFalse(Prover.version().isEmpty());
    }

    @Test
    void proveYieldsHexHashes() {
        try (Prover prover = new Prover()) {
            String proof = prove(prover);
            assertEquals(64, field(proof, "report_hash").length());
            assertEquals(64, field(proof, "inputs_hash").length());
        }
    }

    @Test
    void proveIsReproducible() {
        try (Prover a = new Prover(); Prover b = new Prover()) {
            assertEquals(field(prove(a), "report_hash"), field(prove(b), "report_hash"));
        }
    }

    @Test
    void verifyAcceptsGenuineRejectsTampered() {
        try (Prover prover = new Prover()) {
            String proof = prove(prover);

            String good = prover.command(
                    "{\"cmd\":\"verify\",\"proof\":" + proof + ",\"spec\":" + SPEC + ",\"data\":" + DATA + "}");
            assertEquals("{\"ok\":true,\"valid\":true}", good);

            // Tamper with the report hash: verify recomputes and rejects it.
            String tampered = proof.replaceFirst(
                    "\"report_hash\":\"[0-9a-f]{64}\"",
                    "\"report_hash\":\"" + "0".repeat(64) + "\"");
            String bad = prover.command(
                    "{\"cmd\":\"verify\",\"proof\":" + tampered + ",\"spec\":" + SPEC + ",\"data\":" + DATA + "}");
            assertEquals("{\"ok\":true,\"valid\":false}", bad);
        }
    }

    @Test
    void unknownCommandIsInBandError() {
        try (Prover prover = new Prover()) {
            // An unknown command is not a hard error: the ABI returns a length and
            // the error surfaces in-band as {"ok":false,...} JSON.
            String raw = prover.command("{\"cmd\":\"nope\"}");
            assertTrue(raw.contains("\"ok\":false"), raw);
        }
    }
}
