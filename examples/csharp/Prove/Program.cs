// Prove a backtest in zero knowledge from .NET, then verify the proof.
//
// Reads the shared example inputs (../../specs/momentum.json and
// ../../data/BTCUSDT.csv, next to this project), proves them through the
// command envelope -- the dataset commitment is left to the host -- verifies
// the proof and prints the public journal. With RISC0_DEV_MODE=1 the receipt is
// a fast, unsound placeholder; without it the real prover runs and this takes
// minutes.
//
//   cargo build --release -p wickra-zk-c
//   RISC0_DEV_MODE=1 dotnet run --project examples/csharp/Prove

using System.Globalization;
using System.Text.Json;
using System.Text.Json.Nodes;
using Wickra.Zk;

// bin/<Configuration>/net8.0/ sits three levels below the project, which sits
// under examples/csharp; the shared inputs are under examples.
string examples = Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "../../../../.."));
JsonNode strategy = JsonNode.Parse(File.ReadAllText(Path.Combine(examples, "specs/momentum.json")))!;
var candles = new JsonArray();
foreach (string line in File.ReadAllLines(Path.Combine(examples, "data/BTCUSDT.csv")))
{
    string[] cols = line.Split(',').Select(c => c.Trim()).ToArray();
    if (cols.Length < 6 || !long.TryParse(cols[0], out long time))
    {
        continue; // header
    }
    candles.Add(new JsonObject
    {
        ["time"] = time,
        ["open"] = double.Parse(cols[1], CultureInfo.InvariantCulture),
        ["high"] = double.Parse(cols[2], CultureInfo.InvariantCulture),
        ["low"] = double.Parse(cols[3], CultureInfo.InvariantCulture),
        ["close"] = double.Parse(cols[4], CultureInfo.InvariantCulture),
        ["volume"] = double.Parse(cols[5], CultureInfo.InvariantCulture),
    });
}

using var prover = new Prover();
string proofText = prover.Command(new JsonObject
{
    ["cmd"] = "prove",
    ["spec"] = new JsonObject { ["strategy"] = strategy },
    ["candles"] = candles,
}.ToJsonString());
JsonNode proof = JsonNode.Parse(proofText)!;
if (proof["ok"]?.GetValue<bool>() == false)
{
    Console.Error.WriteLine($"prove failed: {proofText}");
    return 1;
}

string journalText = prover.Command(new JsonObject { ["cmd"] = "verify", ["proof"] = proof.DeepClone() }.ToJsonString());
JsonNode journal = JsonNode.Parse(journalText)!;

Console.WriteLine($"wickra-zk {Prover.Version()}");
Console.WriteLine($"guest_id: {journal["guest_id"]}");
Console.WriteLine($"report_hash: {journal["report_hash"]}");
bool ok = journalText == proof["journal"]!.ToJsonString();
Console.WriteLine(ok ? "verify: valid" : "verify: INVALID");
Console.WriteLine(journal.ToJsonString(new JsonSerializerOptions { WriteIndented = true }));
return ok ? 0 : 1;
