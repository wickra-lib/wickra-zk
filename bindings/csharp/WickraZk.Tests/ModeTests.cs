using System.Text.Json;
using System.Text.Json.Nodes;
using Wickra.Zk;
using Xunit;

namespace WickraZk.Tests;

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
// of the envelope, not the proof system.
public class ModeTests
{
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
            double F(int i) => double.Parse(cols[i], System.Globalization.CultureInfo.InvariantCulture);
            candles.Add(new JsonObject
            {
                ["time"] = time, ["open"] = F(1), ["high"] = F(2), ["low"] = F(3), ["close"] = F(4), ["volume"] = F(5),
            });
        }
        return candles;
    }

    private static JsonObject Command(Prover prover, JsonObject envelope)
    {
        JsonObject response = JsonNode.Parse(prover.Command(envelope.ToJsonString()))!.AsObject();
        if (response.TryGetPropertyValue("ok", out JsonNode? ok) && ok?.GetValue<bool>() == false)
        {
            throw new InvalidOperationException(response["error"]!.GetValue<string>());
        }
        return response;
    }

    [Fact]
    public void HostCommittedAndStatedCommitmentProofsCarryOneJournal()
    {
        RequireDevMode();
        var cases = new SortedDictionary<string, string>(
            JsonSerializer.Deserialize<Dictionary<string, string>>(File.ReadAllText(Path.Combine(Golden, "cases.json")))!);
        Assert.NotEmpty(cases);
        using var prover = new Prover();

        foreach ((string name, string dataset) in cases)
        {
            JsonNode Strategy() => JsonNode.Parse(File.ReadAllText(Path.Combine(Golden, "specs", $"{name}.json")))!;
            JsonArray candles = LoadCandles(dataset);

            // Mode 1: the host commits the candles it is handed.
            JsonObject host = Command(prover, new JsonObject
            {
                ["cmd"] = "prove",
                ["spec"] = new JsonObject { ["strategy"] = Strategy() },
                ["candles"] = candles.DeepClone(),
            });
            // Mode 2: the caller states the commitment up front.
            string commitment = Command(prover, new JsonObject { ["cmd"] = "commit", ["candles"] = candles.DeepClone() })
                ["dataset_commitment"]!.GetValue<string>();
            JsonObject stated = Command(prover, new JsonObject
            {
                ["cmd"] = "prove",
                ["spec"] = new JsonObject { ["strategy"] = Strategy(), ["dataset_commitment"] = commitment },
                ["candles"] = candles.DeepClone(),
            });

            string journal = host["journal"]!.ToJsonString();
            Assert.Equal(journal, stated["journal"]!.ToJsonString());
            Assert.Equal(commitment, stated["journal"]!["dataset_commitment"]!.GetValue<string>());
            Assert.Equal(journal, Command(prover, new JsonObject { ["cmd"] = "verify", ["proof"] = host.DeepClone() }).ToJsonString());
            Assert.Equal(journal, Command(prover, new JsonObject { ["cmd"] = "verify", ["proof"] = stated.DeepClone() }).ToJsonString());
        }
    }
}
