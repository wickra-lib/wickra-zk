package wickra

// Operating-mode equivalence: a proof does not depend on who commits the data.
//
// `prove` runs in two operating modes. The caller can leave the dataset
// commitment to the host, which hashes the candles it is handed, or state it
// up front -- the value `commit` returns, or one carried over from a data
// vendor. The journal must not depend on which: same report_hash, same
// metrics, same dataset_commitment, same guest_id, and both proofs verify to
// that journal. Only the receipt bytes may differ between two runs.
//
// Dev-mode receipts are unsound and fast (init() in zk_test.go sets it); this
// tests the binding's transport of the envelope, not the proof system. The
// golden directory, candle loader and command helper are shared with
// zk_test.go.

import (
	"encoding/json"
	"path/filepath"
	"reflect"
	"sort"
	"testing"
)

func TestHostCommittedAndStatedCommitmentProofsCarryOneJournal(t *testing.T) {
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

	for _, name := range names {
		var strategy json.RawMessage
		readJSON(t, filepath.Join(golden, "specs", name+".json"), &strategy)
		candles := loadCandles(t, golden, cases[name])

		// Mode 1: the host commits the candles it is handed.
		host, err := command(t, p, map[string]any{
			"cmd": "prove", "spec": map[string]any{"strategy": strategy}, "candles": candles,
		})
		if err != "" {
			t.Fatalf("%s: host-committed prove: %s", name, err)
		}
		// Mode 2: the caller states the commitment up front.
		commit, err := command(t, p, map[string]any{"cmd": "commit", "candles": candles})
		if err != "" {
			t.Fatalf("%s: commit: %s", name, err)
		}
		commitment := commit["dataset_commitment"]
		stated, err := command(t, p, map[string]any{
			"cmd":     "prove",
			"spec":    map[string]any{"strategy": strategy, "dataset_commitment": commitment},
			"candles": candles,
		})
		if err != "" {
			t.Fatalf("%s: stated-commitment prove: %s", name, err)
		}

		journal := host["journal"]
		if !reflect.DeepEqual(stated["journal"], journal) {
			t.Fatalf("%s: journals differ between modes:\n host:   %v\n stated: %v", name, journal, stated["journal"])
		}
		if stated["journal"].(map[string]any)["dataset_commitment"] != commitment {
			t.Fatalf("%s: stated commitment not carried", name)
		}
		for mode, proof := range map[string]map[string]any{"host": host, "stated": stated} {
			outputs, err := command(t, p, map[string]any{"cmd": "verify", "proof": proof})
			if err != "" {
				t.Fatalf("%s: verify(%s): %s", name, mode, err)
			}
			if !reflect.DeepEqual(outputs, journal) {
				t.Fatalf("%s: verify(%s) returned a different journal", name, mode)
			}
		}
	}
}
