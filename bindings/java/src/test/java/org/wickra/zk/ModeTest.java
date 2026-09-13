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

// Operating-mode equivalence: a proof does not depend on who commits the data.
//
// `prove` runs in two operating modes. The caller can leave the dataset
// commitment to the host, which hashes the candles it is handed, or state it
// up front -- the value `commit` returns, or one carried over from a data
// vendor. The journal must not depend on which: same report_hash, same
// metrics, same dataset_commitment, same guest_id, and both proofs verify to
// that journal. Only the receipt bytes may differ between two runs.
//
// Dev-mode receipts are unsound and fast; this tests the binding's transport
// of the envelope, not the proof system. The binding carries no JSON library,
// so the journal is cut out of the proof by the same pattern ProverTest uses.
class ModeTest {
    private static final Pattern JOURNAL = Pattern.compile("\"journal\":(\\{\"report_hash\":[^{}]*\\})");
    private static final Pattern IN_BAND_ERROR = Pattern.compile("^\\{\"ok\":false,\"error\":\"(.*)\"\\}$");

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

    private static String journal(String proof, String what) {
        Matcher span = JOURNAL.matcher(proof);
        assertTrue(span.find(), what + ": no journal in " + proof);
        return span.group(1);
    }

    private static String ok(String response, String what) {
        assertFalse(IN_BAND_ERROR.matcher(response).matches(), what + ": " + response);
        return response;
    }

    @Test
    void hostCommittedAndStatedCommitmentProofsCarryOneJournal() throws IOException {
        requireDevMode();
        Path golden = golden();
        try (Prover prover = new Prover()) {
            for (var entry : cases(golden).entrySet()) {
                String name = entry.getKey();
                String strategy = Files.readString(golden.resolve("specs").resolve(name + ".json")).strip();
                String candles = candles(golden, entry.getValue());

                // Mode 1: the host commits the candles it is handed.
                String host = ok(prover.command(
                        "{\"cmd\":\"prove\",\"spec\":{\"strategy\":" + strategy + "},\"candles\":" + candles + "}"),
                        name + ": host-committed prove");
                // Mode 2: the caller states the commitment up front.
                String commitment = string(ok(prover.command("{\"cmd\":\"commit\",\"candles\":" + candles + "}"), name + ": commit"),
                        "dataset_commitment");
                String stated = ok(prover.command(
                        "{\"cmd\":\"prove\",\"spec\":{\"strategy\":" + strategy + ",\"dataset_commitment\":\"" + commitment
                                + "\"},\"candles\":" + candles + "}"),
                        name + ": stated-commitment prove");

                String journal = journal(host, name + ": host");
                assertEquals(journal, journal(stated, name + ": stated"), name + ": journals differ between modes");
                assertEquals(commitment, string(journal, "dataset_commitment"), name);
                assertEquals(journal, ok(prover.command("{\"cmd\":\"verify\",\"proof\":" + host + "}"), name + ": verify(host)"), name);
                assertEquals(journal, ok(prover.command("{\"cmd\":\"verify\",\"proof\":" + stated + "}"), name + ": verify(stated)"), name);
            }
        }
    }
}
