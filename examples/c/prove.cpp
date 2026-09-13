// A minimal C++ example: prove a (strategy, candles) pair through the wickra-zk
// C ABI, print the public journal, then verify the proof and assert it holds.
//
// The prove response is itself a JSON object, so the verify command embeds it
// verbatim as the "proof" value -- no JSON parser is needed on the C++ side.
// The dataset commitment is left to the host: it hashes the candles the same
// way the guest recomputes them.
//
// With RISC0_DEV_MODE=1 the receipt is a fast, unsound placeholder; without it
// the real prover runs and this takes minutes.
#include <iostream>
#include <string>

#include "wickra_zk.hpp"

namespace {
const char *STRATEGY =
    R"({"symbol":"AAA","timeframe":"1h",)"
    R"("indicators":{"ema_fast":{"type":"Ema","params":[3]},)"
    R"("ema_slow":{"type":"Ema","params":[8]}},)"
    R"("entry":{"cross_above":["ema_fast","ema_slow"]},)"
    R"("exit":{"cross_below":["ema_fast","ema_slow"]},)"
    R"("sizing":{"type":"fixed_fraction","fraction":0.95},)"
    R"("costs":{"taker_bps":5,"slippage":{"type":"fixed_bps","bps":2}},)"
    R"("risk":{}})";

// A short V-shaped price path so the fast/slow EMA cross fires at least once.
const char *CANDLES =
    R"([)"
    R"({"time":1700000000,"open":120,"high":121,"low":119,"close":120,"volume":1000},)"
    R"({"time":1700003600,"open":120,"high":121,"low":117,"close":118,"volume":1000},)"
    R"({"time":1700007200,"open":118,"high":119,"low":115,"close":116,"volume":1000},)"
    R"({"time":1700010800,"open":116,"high":117,"low":113,"close":114,"volume":1000},)"
    R"({"time":1700014400,"open":114,"high":115,"low":111,"close":112,"volume":1000},)"
    R"({"time":1700018000,"open":112,"high":113,"low":109,"close":110,"volume":1000},)"
    R"({"time":1700021600,"open":110,"high":111,"low":107,"close":108,"volume":1000},)"
    R"({"time":1700025200,"open":108,"high":113,"low":107,"close":112,"volume":1000},)"
    R"({"time":1700028800,"open":112,"high":117,"low":111,"close":116,"volume":1000},)"
    R"({"time":1700032400,"open":116,"high":121,"low":115,"close":120,"volume":1000},)"
    R"({"time":1700036000,"open":120,"high":125,"low":119,"close":124,"volume":1000},)"
    R"({"time":1700039600,"open":124,"high":129,"low":123,"close":128,"volume":1000}])";
}  // namespace

int main() {
    wickra_zk::Prover prover;

    const std::string proof = prover.command(std::string(R"({"cmd":"prove","spec":{"strategy":)") + STRATEGY +
                                             R"(},"candles":)" + CANDLES + "}");

    std::cout << "wickra-zk " << wickra_zk_version() << "\n";
    // The receipt carries its own "journal" (the committed bytes); the public
    // outputs are the object that opens with report_hash.
    const std::string::size_type at = proof.find(R"("journal":{"report_hash":)");
    if (at == std::string::npos) {
        std::cerr << "prove did not return a journal: " << proof << "\n";
        return 1;
    }
    const std::string::size_type open = at + std::string(R"("journal":)").size();
    const std::string journal = proof.substr(open, proof.find('}', open) - open + 1);
    std::cout << "proof: " << journal << "\n";

    // Verify: the prove response is valid JSON, so it drops straight in as the
    // "proof" value. The outputs come back decoded from the receipt.
    const std::string outputs = prover.command(std::string(R"({"cmd":"verify","proof":)") + proof + "}");
    const bool ok = outputs == journal;
    std::cout << "verify: " << (ok ? "valid" : "INVALID") << "\n";
    if (!ok) {
        std::cerr << "verification did not hold: " << outputs << "\n";
        return 1;
    }
    return 0;
}
