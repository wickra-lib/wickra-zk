package wickra

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

import (
	"encoding/json"
	"math"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"testing"
)

func init() {
	os.Setenv("RISC0_DEV_MODE", "1")
}

func goldenDir(t *testing.T) string {
	t.Helper()
	dir, err := os.Getwd()
	if err != nil {
		t.Fatal(err)
	}
	for i := 0; i < 8; i++ {
		candidate := filepath.Join(dir, "golden")
		if _, err := os.Stat(filepath.Join(candidate, "cases.json")); err == nil {
			return candidate
		}
		dir = filepath.Dir(dir)
	}
	t.Fatal("golden/cases.json not found above the module")
	return ""
}

func readJSON(t *testing.T, path string, into any) {
	t.Helper()
	raw, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}
	if err := json.Unmarshal(raw, into); err != nil {
		t.Fatalf("%s: %v", path, err)
	}
}

type candle struct {
	Time   int64   `json:"time"`
	Open   float64 `json:"open"`
	High   float64 `json:"high"`
	Low    float64 `json:"low"`
	Close  float64 `json:"close"`
	Volume float64 `json:"volume"`
}

func loadCandles(t *testing.T, golden, dataset string) []candle {
	t.Helper()
	raw, err := os.ReadFile(filepath.Join(golden, "data", dataset+".csv"))
	if err != nil {
		t.Fatal(err)
	}
	var out []candle
	for _, line := range strings.Split(string(raw), "\n") {
		cols := strings.Split(strings.TrimSpace(line), ",")
		if len(cols) < 6 {
			continue
		}
		ts, err := strconv.ParseInt(cols[0], 10, 64)
		if err != nil {
			continue // header
		}
		f := func(s string) float64 {
			v, err := strconv.ParseFloat(s, 64)
			if err != nil {
				t.Fatal(err)
			}
			return v
		}
		out = append(out, candle{ts, f(cols[1]), f(cols[2]), f(cols[3]), f(cols[4]), f(cols[5])})
	}
	return out
}

// command sends an envelope and decodes the response; an in-band error is
// returned as a string so the caller can assert on it.
func command(t *testing.T, p *Prover, envelope map[string]any) (map[string]any, string) {
	t.Helper()
	cmd, err := json.Marshal(envelope)
	if err != nil {
		t.Fatal(err)
	}
	raw, err := p.Command(string(cmd))
	if err != nil {
		t.Fatal(err)
	}
	var response map[string]any
	if err := json.Unmarshal([]byte(raw), &response); err != nil {
		t.Fatalf("not a JSON object: %s", raw)
	}
	if ok, present := response["ok"]; present && ok == false {
		return nil, response["error"].(string)
	}
	return response, ""
}

func TestGoldenCasesProveToTheBlessedJournal(t *testing.T) {
	golden := goldenDir(t)
	var cases map[string]string
	readJSON(t, filepath.Join(golden, "cases.json"), &cases)
	names := make([]string, 0, len(cases))
	for name := range cases {
		names = append(names, name)
	}
	sort.Strings(names)
	if len(names) == 0 {
		t.Fatal("no golden cases; this would test nothing")
	}

	p := New()
	defer p.Close()
	version, _ := command(t, p, map[string]any{"cmd": "version"})

	for _, name := range names {
		var strategy json.RawMessage
		readJSON(t, filepath.Join(golden, "specs", name+".json"), &strategy)
		var expected struct {
			ReportHash string  `json:"report_hash"`
			Sharpe     float64 `json:"sharpe"`
			Pnl        float64 `json:"pnl"`
			NTrades    float64 `json:"n_trades"`
		}
		readJSON(t, filepath.Join(golden, "expected", name+".json"), &expected)
		candles := loadCandles(t, golden, cases[name])

		commit, inband := command(t, p, map[string]any{"cmd": "commit", "candles": candles})
		if inband != "" {
			t.Fatalf("%s: commit: %s", name, inband)
		}
		proof, inband := command(t, p, map[string]any{
			"cmd": "prove", "spec": map[string]any{"strategy": strategy}, "candles": candles,
		})
		if inband != "" {
			t.Fatalf("%s: prove: %s", name, inband)
		}
		journal := proof["journal"].(map[string]any)

		if journal["report_hash"] != expected.ReportHash {
			t.Fatalf("%s: report_hash %v, blessed %s", name, journal["report_hash"], expected.ReportHash)
		}
		if journal["n_trades"] != expected.NTrades {
			t.Fatalf("%s: n_trades %v, blessed %v", name, journal["n_trades"], expected.NTrades)
		}
		if math.Abs(journal["sharpe"].(float64)-expected.Sharpe) >= 1e-8 {
			t.Fatalf("%s: sharpe %v, blessed %v", name, journal["sharpe"], expected.Sharpe)
		}
		if math.Abs(journal["pnl"].(float64)-expected.Pnl) >= 1e-8 {
			t.Fatalf("%s: pnl %v, blessed %v", name, journal["pnl"], expected.Pnl)
		}
		if journal["dataset_commitment"] != commit["dataset_commitment"] {
			t.Fatalf("%s: the journal is not bound to the candles", name)
		}
		if journal["guest_id"] != version["guest_id"] {
			t.Fatalf("%s: guest_id %v, host pins %v", name, journal["guest_id"], version["guest_id"])
		}

		outputs, inband := command(t, p, map[string]any{"cmd": "verify", "proof": proof})
		if inband != "" {
			t.Fatalf("%s: verify: %s", name, inband)
		}
		got, _ := json.Marshal(outputs)
		want, _ := json.Marshal(journal)
		if string(got) != string(want) {
			t.Fatalf("%s: verify returned %s, journal is %s", name, got, want)
		}

		journal["report_hash"] = strings.Repeat("f", 64)
		if _, inband := command(t, p, map[string]any{"cmd": "verify", "proof": proof}); !strings.Contains(inband, "verify") {
			t.Fatalf("%s: a proof file whose journal lies must be refused, got %q", name, inband)
		}
	}
}

func TestAStatedCommitmentIsHeldTo(t *testing.T) {
	golden := goldenDir(t)
	var cases map[string]string
	readJSON(t, filepath.Join(golden, "cases.json"), &cases)
	names := make([]string, 0, len(cases))
	for name := range cases {
		names = append(names, name)
	}
	sort.Strings(names)
	var strategy json.RawMessage
	readJSON(t, filepath.Join(golden, "specs", names[0]+".json"), &strategy)
	candles := loadCandles(t, golden, cases[names[0]])

	p := New()
	defer p.Close()
	_, inband := command(t, p, map[string]any{
		"cmd":     "prove",
		"spec":    map[string]any{"strategy": strategy, "dataset_commitment": strings.Repeat("0", 64)},
		"candles": candles,
	})
	if !strings.Contains(inband, "commitment mismatch") {
		t.Fatalf("expected a commitment mismatch, got %q", inband)
	}
}

func TestVersionIsReportedTwoWaysAndTheyAgree(t *testing.T) {
	if Version() == "" {
		t.Fatal("empty version")
	}
	p := New()
	defer p.Close()
	response, inband := command(t, p, map[string]any{"cmd": "version"})
	if inband != "" {
		t.Fatal(inband)
	}
	if response["version"] != Version() {
		t.Fatalf("command reports %v, Version() reports %s", response["version"], Version())
	}
}

func TestAMalformedEnvelopeIsAnInBandError(t *testing.T) {
	p := New()
	defer p.Close()
	raw, err := p.Command("not json")
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(raw, `"ok":false`) {
		t.Fatalf("expected an in-band error, got %s", raw)
	}
}
