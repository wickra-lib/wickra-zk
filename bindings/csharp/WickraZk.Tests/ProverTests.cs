using System.Text.Json;
using Wickra.Proof;
using Xunit;

namespace WickraZk.Tests;

public class ProverTests
{
    private static object Strategy() => new
    {
        symbol = "BTCUSDT",
        timeframe = "1h",
        indicators = new
        {
            ema_fast = new { type = "Ema", @params = new[] { 5 } },
            ema_slow = new { type = "Ema", @params = new[] { 15 } },
        },
        entry = new { cross_above = new[] { "ema_fast", "ema_slow" } },
        exit = new { cross_below = new[] { "ema_fast", "ema_slow" } },
        sizing = new { type = "fixed_fraction", fraction = 0.95 },
        costs = new { taker_bps = 5, slippage = new { type = "fixed_bps", bps = 2 } },
        risk = new { trailing_stop_pct = 5.0 },
    };

    private static object[] Candles()
    {
        var list = new List<object>();
        for (int i = 0; i < 40; i++)
        {
            double b = 100.0 + Math.Sin(i * 0.4) * 8.0;
            list.Add(new { time = 1_700_000_000 + i * 3600, open = b, high = b + 1.0, low = b - 1.0, close = b + 0.5, volume = 1000.0 });
        }
        return [.. list];
    }

    private static object Spec() => new { strategy = Strategy(), dataset_ref = "BTCUSDT/1h/test" };

    private static object Data() => new Dictionary<string, object[]> { ["BTCUSDT"] = Candles() };

    private static JsonElement Prove(Prover prover)
    {
        string cmd = JsonSerializer.Serialize(new { cmd = "prove", spec = Spec(), data = Data() });
        return JsonDocument.Parse(prover.Command(cmd)).RootElement.Clone();
    }

    [Fact]
    public void Version_IsNonEmpty()
    {
        Assert.False(string.IsNullOrEmpty(Prover.Version()));
    }

    [Fact]
    public void Prove_YieldsHexHashes()
    {
        using var prover = new Prover();
        JsonElement proof = Prove(prover);
        Assert.Equal(64, proof.GetProperty("report_hash").GetString()!.Length);
        Assert.Equal(64, proof.GetProperty("inputs_hash").GetString()!.Length);
    }

    [Fact]
    public void Prove_IsReproducible()
    {
        using var a = new Prover();
        using var b = new Prover();
        Assert.Equal(
            Prove(a).GetProperty("report_hash").GetString(),
            Prove(b).GetProperty("report_hash").GetString());
    }

    [Fact]
    public void Verify_AcceptsGenuineRejectsTampered()
    {
        using var prover = new Prover();
        JsonElement proof = Prove(prover);

        string good = prover.Command(JsonSerializer.Serialize(new { cmd = "verify", proof, spec = Spec(), data = Data() }));
        Assert.Equal("{\"ok\":true,\"valid\":true}", good);

        // Tamper with the report hash: verify recomputes and rejects it.
        var tampered = JsonSerializer.Deserialize<Dictionary<string, JsonElement>>(proof.GetRawText())!;
        tampered["report_hash"] = JsonSerializer.SerializeToElement(new string('0', 64));
        string bad = prover.Command(JsonSerializer.Serialize(new { cmd = "verify", proof = tampered, spec = Spec(), data = Data() }));
        Assert.Equal("{\"ok\":true,\"valid\":false}", bad);
    }

    [Fact]
    public void UnknownCommand_IsInBandError()
    {
        using var prover = new Prover();
        // An unknown command is not a hard error: the ABI returns a length and the
        // error surfaces in-band as {"ok":false,...} JSON.
        string raw = prover.Command("{\"cmd\":\"nope\"}");
        Assert.Contains("\"ok\":false", raw);
    }
}
