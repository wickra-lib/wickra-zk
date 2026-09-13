// Prove a backtest in zero knowledge from Go, then verify the proof.
//
// Reads the shared example inputs (../specs/momentum.json and
// ../data/BTCUSDT.csv), proves them through the command envelope -- the
// dataset commitment is left to the host -- verifies the proof and prints the
// public journal. With RISC0_DEV_MODE=1 the receipt is a fast, unsound
// placeholder; without it the real prover runs and this takes minutes.
//
//	RISC0_DEV_MODE=1 go run .
package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"

	wickra "github.com/wickra-lib/wickra-zk-go"
)

type candle struct {
	Time   int64   `json:"time"`
	Open   float64 `json:"open"`
	High   float64 `json:"high"`
	Low    float64 `json:"low"`
	Close  float64 `json:"close"`
	Volume float64 `json:"volume"`
}

func loadCandles(path string) ([]candle, error) {
	raw, err := os.ReadFile(path)
	if err != nil {
		return nil, err
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
		var f [5]float64
		for i := range f {
			if f[i], err = strconv.ParseFloat(cols[i+1], 64); err != nil {
				return nil, err
			}
		}
		out = append(out, candle{ts, f[0], f[1], f[2], f[3], f[4]})
	}
	return out, nil
}

func main() {
	_, here, _, _ := runtime.Caller(0)
	dir := filepath.Dir(here)
	strategy, err := os.ReadFile(filepath.Join(dir, "../specs/momentum.json"))
	if err != nil {
		panic(err)
	}
	candles, err := loadCandles(filepath.Join(dir, "../data/BTCUSDT.csv"))
	if err != nil {
		panic(err)
	}

	p := wickra.New()
	defer p.Close()

	cmd, _ := json.Marshal(map[string]any{
		"cmd":     "prove",
		"spec":    map[string]any{"strategy": json.RawMessage(strategy)},
		"candles": candles,
	})
	proofText, err := p.Command(string(cmd))
	if err != nil {
		panic(err)
	}
	var proof struct {
		Journal json.RawMessage `json:"journal"`
		Error   string          `json:"error"`
	}
	if err := json.Unmarshal([]byte(proofText), &proof); err != nil || proof.Error != "" {
		panic("prove failed: " + proofText)
	}

	cmd, _ = json.Marshal(map[string]any{"cmd": "verify", "proof": json.RawMessage(proofText)})
	journalText, err := p.Command(string(cmd))
	if err != nil {
		panic(err)
	}
	var journal struct {
		ReportHash string `json:"report_hash"`
		GuestID    string `json:"guest_id"`
	}
	if err := json.Unmarshal([]byte(journalText), &journal); err != nil {
		panic(err)
	}

	fmt.Printf("wickra-zk %s\n", wickra.Version())
	fmt.Printf("guest_id: %s\n", journal.GuestID)
	fmt.Printf("report_hash: %s\n", journal.ReportHash)
	if journalText == string(proof.Journal) {
		fmt.Println("verify: valid")
	} else {
		fmt.Println("verify: INVALID")
	}
	fmt.Println(journalText)
}
