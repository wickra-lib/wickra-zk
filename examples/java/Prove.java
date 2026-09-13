// Prove a backtest in zero knowledge from Java, then verify the proof.
//
// Reads the shared example inputs (../specs/momentum.json and
// ../data/BTCUSDT.csv, relative to this file's directory, passed as the first
// argument), proves them through the command envelope -- the dataset
// commitment is left to the host -- verifies the proof and prints the public
// journal. With RISC0_DEV_MODE=1 the receipt is a fast, unsound placeholder;
// without it the real prover runs and this takes minutes.
//
//   cargo build -p wickra-zk-c --release
//   mvn -f bindings/java/pom.xml -q package -DskipTests
//   javac -cp bindings/java/target/classes examples/java/Prove.java -d examples/java/out
//   RISC0_DEV_MODE=1 java --enable-native-access=ALL-UNNAMED \
//        -Dnative.lib.dir=target/release \
//        -cp "bindings/java/target/classes:examples/java/out" Prove examples
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import org.wickra.zk.Prover;

public final class Prove {
    private static String candles(Path csv) throws IOException {
        List<String> rows = new ArrayList<>();
        for (String raw : Files.readAllLines(csv)) {
            String[] cols = raw.trim().split(",");
            if (cols.length < 6 || !cols[0].trim().matches("\\d+")) {
                continue; // header
            }
            rows.add("{\"time\":" + cols[0].trim() + ",\"open\":" + cols[1].trim() + ",\"high\":" + cols[2].trim()
                    + ",\"low\":" + cols[3].trim() + ",\"close\":" + cols[4].trim() + ",\"volume\":" + cols[5].trim() + "}");
        }
        return "[" + String.join(",", rows) + "]";
    }

    private static String field(String json, String key) {
        Matcher m = Pattern.compile("\"" + key + "\":\"([0-9a-f]{64})\"").matcher(json);
        if (!m.find()) {
            throw new IllegalStateException("missing " + key + " in " + json);
        }
        return m.group(1);
    }

    public static void main(String[] args) throws IOException {
        Path examples = Path.of(args.length > 0 ? args[0] : "examples");
        String strategy = Files.readString(examples.resolve("specs/momentum.json")).strip();
        String candles = candles(examples.resolve("data/BTCUSDT.csv"));

        try (Prover prover = new Prover()) {
            String proof = prover.command(
                    "{\"cmd\":\"prove\",\"spec\":{\"strategy\":" + strategy + "},\"candles\":" + candles + "}");
            if (proof.startsWith("{\"ok\":false")) {
                throw new IllegalStateException("prove failed: " + proof);
            }
            // The receipt carries its own "journal" (the committed bytes); the
            // public outputs are the object that opens with report_hash.
            Matcher span = Pattern.compile("\"journal\":(\\{\"report_hash\":[^{}]*\\})").matcher(proof);
            if (!span.find()) {
                throw new IllegalStateException("no journal in " + proof);
            }
            String journal = prover.command("{\"cmd\":\"verify\",\"proof\":" + proof + "}");

            System.out.println("wickra-zk " + Prover.version());
            System.out.println("guest_id: " + field(journal, "guest_id"));
            System.out.println("report_hash: " + field(journal, "report_hash"));
            System.out.println(journal.equals(span.group(1)) ? "verify: valid" : "verify: INVALID");
            System.out.println(journal);
        }
    }
}
