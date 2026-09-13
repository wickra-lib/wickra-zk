package org.wickra.zk;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.TreeMap;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import org.junit.jupiter.api.Test;

// Cross-language golden parity through the command envelope, in dev-mode.
//
// Each golden case is proven through the binding without a stated commitment,
// and the journal must carry the blessed report_hash and metrics -- the same
// determinism chain crates/wickra-zk-host/tests/golden.rs pins natively. The
// proof then verifies to the same journal, a proof file whose journal disagrees
// with its receipt is refused, and a stated commitment that is not the hash of
// the candles is refused before the zkVM runs.
//
// Dev-mode receipts are unsound and fast; this tests the binding's transport
// of the envelope, not the proof system. The real prover runs nightly. The
// binding carries no JSON library, so fields are read back by pattern; the
// journal is flat, so its object is one brace pair.
class ProverTest {
    // The receipt carries its own "journal" (the committed bytes); the
    // persisted public outputs are the object that opens with report_hash.
    private static final Pattern JOURNAL = Pattern.compile("\"journal\":(\\{\"report_hash\":[^{}]*\\})");
    private static final Pattern IN_BAND_ERROR = Pattern.compile("^\\{\"ok\":false,\"error\":\"(.*)\"\\}$");

    // The prover reads RISC0_DEV_MODE through getenv; a JVM cannot set it for
    // itself, so it must come from the process environment (surefire passes
    // it through), and a test that would otherwise run the real prover for
    // minutes says so.
    private static void requireDevMode() {
        assertEquals("1", System.getenv("RISC0_DEV_MODE"),
                "set RISC0_DEV_MODE=1: this test drives the envelope, not the real prover");
    }

    private static Path golden() {
        Path dir = Path.of("").toAbsolutePath();
        for (int i = 0; i < 10 && dir != null; i++) {
            Path candidate = dir.resolve("golden");
            if (Files.isRegularFile(candidate.resolve("cases.json"))) {
                return candidate;
            }
            dir = dir.getParent();
        }
        throw new IllegalStateException("golden/cases.json not found above the module");
    }

    private static TreeMap<String, String> cases(Path golden) throws IOException {
        var out = new TreeMap<String, String>();
        Matcher m = Pattern.compile("\"([^\"]+)\"\\s*:\\s*\"([^\"]+)\"").matcher(Files.readString(golden.resolve("cases.json")));
        while (m.find()) {
            out.put(m.group(1), m.group(2));
        }
        assertFalse(out.isEmpty(), "no golden cases; this would test nothing");
        return out;
    }

    private static String candles(Path golden, String dataset) throws IOException {
        List<String> rows = new ArrayList<>();
        for (String raw : Files.readAllLines(golden.resolve("data").resolve(dataset + ".csv"))) {
            String[] cols = raw.trim().split(",");
            if (cols.length < 6 || !cols[0].trim().matches("\\d+")) {
                continue; // header
            }
            rows.add("{\"time\":" + cols[0].trim() + ",\"open\":" + cols[1].trim() + ",\"high\":" + cols[2].trim()
                    + ",\"low\":" + cols[3].trim() + ",\"close\":" + cols[4].trim() + ",\"volume\":" + cols[5].trim() + "}");
        }
        return "[" + String.join(",", rows) + "]";
    }

    private static String string(String json, String key) {
        Matcher m = Pattern.compile("\"" + key + "\"\\s*:\\s*\"([^\"]*)\"").matcher(json);
        assertTrue(m.find(), "missing " + key + " in " + json);
        return m.group(1);
    }

    private static double number(String json, String key) {
        Matcher m = Pattern.compile("\"" + key + "\"\\s*:\\s*(-?[0-9.eE+-]+)").matcher(json);
        assertTrue(m.find(), "missing " + key + " in " + json);
        return Double.parseDouble(m.group(1));
    }

    private static String inBandError(String response) {
        Matcher m = IN_BAND_ERROR.matcher(response);
        return m.matches() ? m.group(1) : null;
    }

    @Test
    void goldenCasesProveToTheBlessedJournal() throws IOException {
        requireDevMode();
        Path golden = golden();
        try (Prover prover = new Prover()) {
            String guestId = string(prover.command("{\"cmd\":\"version\"}"), "guest_id");
            for (var entry : cases(golden).entrySet()) {
                String name = entry.getKey();
                String strategy = Files.readString(golden.resolve("specs").resolve(name + ".json")).strip();
                String expected = Files.readString(golden.resolve("expected").resolve(name + ".json")).strip();
                String candles = candles(golden, entry.getValue());

                String commit = prover.command("{\"cmd\":\"commit\",\"candles\":" + candles + "}");
                assertEquals(null, inBandError(commit), name);
                String proof = prover.command(
                        "{\"cmd\":\"prove\",\"spec\":{\"strategy\":" + strategy + "},\"candles\":" + candles + "}");
                assertEquals(null, inBandError(proof), name);
                Matcher span = JOURNAL.matcher(proof);
                assertTrue(span.find(), name + ": no journal in " + proof);
                String journal = span.group(1);

                assertEquals(string(expected, "report_hash"), string(journal, "report_hash"), name);
                assertEquals(number(expected, "n_trades"), number(journal, "n_trades"), name);
                assertTrue(Math.abs(number(journal, "sharpe") - number(expected, "sharpe")) < 1e-8, name + ": sharpe");
                assertTrue(Math.abs(number(journal, "pnl") - number(expected, "pnl")) < 1e-8, name + ": pnl");
                assertEquals(string(commit, "dataset_commitment"), string(journal, "dataset_commitment"), name);
                assertEquals(guestId, string(journal, "guest_id"), name);

                String outputs = prover.command("{\"cmd\":\"verify\",\"proof\":" + proof + "}");
                assertEquals(journal, outputs, name);

                String lying = proof.substring(0, span.start(1))
                        + journal.replaceFirst("\"report_hash\":\"[0-9a-f]{64}\"", "\"report_hash\":\"" + "f".repeat(64) + "\"")
                        + proof.substring(span.end(1));
                String refused = inBandError(prover.command("{\"cmd\":\"verify\",\"proof\":" + lying + "}"));
                assertTrue(refused != null && refused.contains("verify"), name + ": " + refused);
            }
        }
    }

    @Test
    void aStatedCommitmentIsHeldTo() throws IOException {
        Path golden = golden();
        var first = cases(golden).firstEntry();
        String strategy = Files.readString(golden.resolve("specs").resolve(first.getKey() + ".json")).strip();
        try (Prover prover = new Prover()) {
            String refused = inBandError(prover.command(
                    "{\"cmd\":\"prove\",\"spec\":{\"strategy\":" + strategy + ",\"dataset_commitment\":\"" + "0".repeat(64)
                            + "\"},\"candles\":" + candles(golden, first.getValue()) + "}"));
            assertTrue(refused != null && refused.contains("commitment mismatch"), String.valueOf(refused));
        }
    }

    @Test
    void versionIsReportedTwoWaysAndTheyAgree() {
        assertFalse(Prover.version().isEmpty());
        try (Prover prover = new Prover()) {
            assertEquals(Prover.version(), string(prover.command("{\"cmd\":\"version\"}"), "version"));
        }
    }

    @Test
    void aMalformedEnvelopeIsAnInBandError() {
        try (Prover prover = new Prover()) {
            assertTrue(inBandError(prover.command("not json")) != null);
        }
    }
}
