using System.Text.Json;
using System.Text.Json.Nodes;
using Wickra.Zk;
using Xunit;

namespace WickraZk.Tests;

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
// of the envelope, not the proof system. The real prover runs nightly.
public class ProverTests
{
    // The prover reads RISC0_DEV_MODE through getenv. .NET's
    // Environment.SetEnvironmentVariable does not reach that on Unix (it edits
    // a managed copy), so the variable must come from the process environment,
    // and a test that would otherwise run the real prover for minutes says so.
    private static void RequireDevMode()
    {
        Assert.True(
            Environment.GetEnvironmentVariable("RISC0_DEV_MODE") == "1",
            "set RISC0_DEV_MODE=1: this test drives the envelope, not the real prover");
    }

    private static string GoldenDir()
    {
        string? dir = AppContext.BaseDirectory;
        for (int i = 0; i < 10 && dir is not null; i++)
        {
            string candidate = Path.Combine(dir, "golden");
            if (File.Exists(Path.Combine(candidate, "cases.json")))
            {
                return candidate;
            }
            dir = Path.GetDirectoryName(dir);
        }
        throw new FileNotFoundException("golden/cases.json not found above the test assembly");
    }

    private static readonly string Golden = GoldenDir();

    private static SortedDictionary<string, string> Cases() =>
        new(JsonSerializer.Deserialize<Dictionary<string, string>>(File.ReadAllText(Path.Combine(Golden, "cases.json")))!);

    private static JsonArray LoadCandles(string dataset)
    {
        var candles = new JsonArray();
        foreach (string line in File.ReadAllLines(Path.Combine(Golden, "data", $"{dataset}.csv")))
        {
            string[] cols = line.Split(',').Select(c => c.Trim()).ToArray();
            if (cols.Length < 6 || !long.TryParse(cols[0], out long time))
            {
                continue; // header
            }
            candles.Add(new JsonObject
            {
                ["time"] = time,
                ["open"] = double.Parse(cols[1], System.Globalization.CultureInfo.InvariantCulture),
                ["high"] = double.Parse(cols[2], System.Globalization.CultureInfo.InvariantCulture),
                ["low"] = double.Parse(cols[3], System.Globalization.CultureInfo.InvariantCulture),
                ["close"] = double.Parse(cols[4], System.Globalization.CultureInfo.InvariantCulture),
                ["volume"] = double.Parse(cols[5], System.Globalization.CultureInfo.InvariantCulture),
            });
        }
        return candles;
    }

    // Sends an envelope and returns the response; an in-band error is returned
    // as its message so a test can assert on it.
    private static (JsonNode? Response, string? Error) Command(Prover prover, JsonObject envelope)
    {
        JsonNode response = JsonNode.Parse(prover.Command(envelope.ToJsonString()))!;
        if (response is JsonObject obj && obj.TryGetPropertyValue("ok", out JsonNode? ok) && ok?.GetValue<bool>() == false)
        {
            return (null, obj["error"]!.GetValue<string>());
        }
        return (response, null);
    }

    private static JsonNode Strategy(string name) => JsonNode.Parse(File.ReadAllText(Path.Combine(Golden, "specs", $"{name}.json")))!;

    [Fact]
    public void GoldenCases_ProveToTheBlessedJournal()
    {
        RequireDevMode();
        var cases = Cases();
        Assert.NotEmpty(cases);
        using var prover = new Prover();
        string guestId = Command(prover, new JsonObject { ["cmd"] = "version" }).Response!["guest_id"]!.GetValue<string>();

        foreach ((string name, string dataset) in cases)
        {
            JsonNode expected = JsonNode.Parse(File.ReadAllText(Path.Combine(Golden, "expected", $"{name}.json")))!;
            JsonArray candles = LoadCandles(dataset);

            (JsonNode? commit, string? commitError) = Command(prover, new JsonObject { ["cmd"] = "commit", ["candles"] = candles.DeepClone() });
            Assert.True(commitError is null, $"{name}: commit: {commitError}");

            (JsonNode? proof, string? proveError) = Command(prover, new JsonObject
            {
                ["cmd"] = "prove",
                ["spec"] = new JsonObject { ["strategy"] = Strategy(name) },
                ["candles"] = candles.DeepClone(),
            });
            Assert.True(proveError is null, $"{name}: prove: {proveError}");
            JsonObject journal = proof!["journal"]!.AsObject();

            Assert.Equal(expected["report_hash"]!.GetValue<string>(), journal["report_hash"]!.GetValue<string>());
            Assert.Equal(expected["n_trades"]!.GetValue<long>(), journal["n_trades"]!.GetValue<long>());
            Assert.True(Math.Abs(journal["sharpe"]!.GetValue<double>() - expected["sharpe"]!.GetValue<double>()) < 1e-8, $"{name}: sharpe");
            Assert.True(Math.Abs(journal["pnl"]!.GetValue<double>() - expected["pnl"]!.GetValue<double>()) < 1e-8, $"{name}: pnl");
            Assert.Equal(commit!["dataset_commitment"]!.GetValue<string>(), journal["dataset_commitment"]!.GetValue<string>());
            Assert.Equal(guestId, journal["guest_id"]!.GetValue<string>());

            (JsonNode? outputs, string? verifyError) = Command(prover, new JsonObject { ["cmd"] = "verify", ["proof"] = proof.DeepClone() });
            Assert.True(verifyError is null, $"{name}: verify: {verifyError}");
            Assert.Equal(journal.ToJsonString(), outputs!.ToJsonString());

            JsonNode lying = proof.DeepClone();
            lying["journal"]!["report_hash"] = new string('f', 64);
            (_, string? lyingError) = Command(prover, new JsonObject { ["cmd"] = "verify", ["proof"] = lying });
            Assert.Contains("verify", lyingError ?? "", StringComparison.Ordinal);
        }
    }

    [Fact]
    public void AStatedCommitment_IsHeldTo()
    {
        (string name, string dataset) = Cases().First();
        using var prover = new Prover();
        (_, string? error) = Command(prover, new JsonObject
        {
            ["cmd"] = "prove",
            ["spec"] = new JsonObject { ["strategy"] = Strategy(name), ["dataset_commitment"] = new string('0', 64) },
            ["candles"] = LoadCandles(dataset),
        });
        Assert.Contains("commitment mismatch", error ?? "", StringComparison.Ordinal);
    }

    [Fact]
    public void Version_IsReportedTwoWaysAndTheyAgree()
    {
        Assert.False(string.IsNullOrEmpty(Prover.Version()));
        using var prover = new Prover();
        (JsonNode? response, _) = Command(prover, new JsonObject { ["cmd"] = "version" });
        Assert.Equal(Prover.Version(), response!["version"]!.GetValue<string>());
    }

    [Fact]
    public void AMalformedEnvelope_IsAnInBandError()
    {
        using var prover = new Prover();
        Assert.Contains("\"ok\":false", prover.Command("not json"), StringComparison.Ordinal);
    }
}
