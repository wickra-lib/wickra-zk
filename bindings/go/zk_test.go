package wickra

import (
	"encoding/json"
	"math"
	"strings"
	"testing"
)

const strategy = `{"symbol":"BTCUSDT","timeframe":"1h",` +
	`"indicators":{"ema_fast":{"type":"Ema","params":[5]},"ema_slow":{"type":"Ema","params":[15]}},` +
	`"entry":{"cross_above":["ema_fast","ema_slow"]},"exit":{"cross_below":["ema_fast","ema_slow"]},` +
	`"sizing":{"type":"fixed_fraction","fraction":0.95},` +
	`"costs":{"taker_bps":5,"slippage":{"type":"fixed_bps","bps":2}},` +
	`"risk":{"trailing_stop_pct":5.0}}`

func candles() []map[string]float64 {
	out := make([]map[string]float64, 0, 40)
	for i := 0; i < 40; i++ {
		base := 100.0 + math.Sin(float64(i)*0.4)*8.0
		out = append(out, map[string]float64{
			"time": float64(1_700_000_000 + i*3600), "open": base,
			"high": base + 1.0, "low": base - 1.0, "close": base + 0.5, "volume": 1000.0,
		})
	}
	return out
}

func spec() map[string]any {
	return map[string]any{"strategy": json.RawMessage(strategy), "dataset_ref": "BTCUSDT/1h/test"}
}

func data() map[string]any {
	return map[string]any{"BTCUSDT": candles()}
}

func prove(t *testing.T, p *Prover) map[string]any {
	t.Helper()
	cmd, err := json.Marshal(map[string]any{"cmd": "prove", "spec": spec(), "data": data()})
	if err != nil {
		t.Fatal(err)
	}
	raw, err := p.Command(string(cmd))
	if err != nil {
		t.Fatal(err)
	}
	var proof map[string]any
	if err := json.Unmarshal([]byte(raw), &proof); err != nil {
		t.Fatal(err)
	}
	return proof
}

func TestVersion(t *testing.T) {
	if Version() == "" {
		t.Fatal("empty version")
	}
}

func TestProveYieldsHashes(t *testing.T) {
	p := New()
	defer p.Close()
	proof := prove(t, p)
	if h, _ := proof["report_hash"].(string); len(h) != 64 {
		t.Fatalf("expected a 64-hex report_hash, got %q", proof["report_hash"])
	}
	if h, _ := proof["inputs_hash"].(string); len(h) != 64 {
		t.Fatalf("expected a 64-hex inputs_hash, got %q", proof["inputs_hash"])
	}
}

func TestProveIsReproducible(t *testing.T) {
	a := prove(t, New())
	b := prove(t, New())
	if a["report_hash"] != b["report_hash"] {
		t.Fatalf("report_hash not reproducible: %v vs %v", a["report_hash"], b["report_hash"])
	}
}

func TestVerifyAcceptsGenuineRejectsTampered(t *testing.T) {
	p := New()
	defer p.Close()
	proof := prove(t, p)

	good, err := verify(p, proof)
	if err != nil {
		t.Fatal(err)
	}
	if good != `{"ok":true,"valid":true}` {
		t.Fatalf("expected a valid verdict, got %s", good)
	}

	tampered := make(map[string]any, len(proof))
	for k, v := range proof {
		tampered[k] = v
	}
	tampered["report_hash"] = strings.Repeat("0", 64)
	bad, err := verify(p, tampered)
	if err != nil {
		t.Fatal(err)
	}
	if bad != `{"ok":true,"valid":false}` {
		t.Fatalf("expected an invalid verdict, got %s", bad)
	}
}

func verify(p *Prover, proof map[string]any) (string, error) {
	cmd, err := json.Marshal(map[string]any{"cmd": "verify", "proof": proof, "spec": spec(), "data": data()})
	if err != nil {
		return "", err
	}
	return p.Command(string(cmd))
}

func TestUnknownCommandIsInBandError(t *testing.T) {
	p := New()
	defer p.Close()
	raw, err := p.Command(`{"cmd":"nope"}`)
	if err != nil {
		t.Fatalf("unexpected hard error: %v", err)
	}
	if !strings.Contains(raw, `"ok":false`) {
		t.Fatalf("expected an in-band error, got: %s", raw)
	}
}
