// Optional C++ convenience layer over the wickra-zk C ABI (`wickra_zk.h`).
//
// The C ABI hands out a raw handle that must be released exactly once with
// `wickra_zk_free`, and its command entry point writes into a caller-owned
// buffer and returns the length it wanted. Both are easy to get subtly wrong in
// C++, and neither needs to be: this wraps the handle in a move-only RAII owner
// and the command in a std::string round trip that grows the buffer once if the
// first attempt was too small.
//
//     #include "wickra_zk.hpp"
//
//     wickra_zk::Prover p;
//     auto out = p.command(R"({"cmd":"version"})");
//
// Header-only, and adds no runtime cost beyond the C calls themselves.

#ifndef WICKRA_ZK_HPP
#define WICKRA_ZK_HPP

#include "wickra_zk.h"

#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

namespace wickra_zk {

/// Move-only owner of a `WickraZk *`. Frees exactly once, at scope exit.
class Prover {
 public:
  Prover() : handle_(wickra_zk_new()) {
    if (handle_ == nullptr) {
      throw std::runtime_error("wickra_zk_new returned null");
    }
  }

  ~Prover() {
    if (handle_ != nullptr) {
      wickra_zk_free(handle_);
    }
  }

  Prover(const Prover &) = delete;
  Prover &operator=(const Prover &) = delete;

  Prover(Prover &&other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }

  Prover &operator=(Prover &&other) noexcept {
    if (this != &other) {
      if (handle_ != nullptr) {
        wickra_zk_free(handle_);
      }
      handle_ = other.handle_;
      other.handle_ = nullptr;
    }
    return *this;
  }

  /// The raw handle, for calling the C ABI directly. Ownership stays here.
  WickraZk *get() const noexcept { return handle_; }

  /// Send one command envelope and return the response.
  ///
  /// The C entry point returns the length it needed. A negative value is an
  /// error code from the header (WICKRA_ZK_ERR_*); a value larger than the
  /// buffer means the response did not fit, so it is called again with a
  /// buffer of exactly that size. Two attempts at most: the second is told the
  /// true length rather than guessing.
  ///
  /// A domain error -- a bad spec, a receipt that does not verify -- is not an
  /// exception here. It arrives as `{"ok":false,...}` in the returned string,
  /// which is the same contract every other binding sees.
  std::string command(const std::string &cmd_json) const {
    std::vector<char> buf(8192);
    int32_t needed = wickra_zk_command(handle_, cmd_json.c_str(), buf.data(), buf.size());
    if (needed < 0) {
      throw std::runtime_error("wickra_zk_command failed with code " + std::to_string(needed));
    }
    if (static_cast<size_t>(needed) >= buf.size()) {
      buf.assign(static_cast<size_t>(needed) + 1, '\0');
      needed = wickra_zk_command(handle_, cmd_json.c_str(), buf.data(), buf.size());
      if (needed < 0) {
        throw std::runtime_error("wickra_zk_command failed with code " + std::to_string(needed));
      }
    }
    return std::string(buf.data(), static_cast<size_t>(needed));
  }

 private:
  WickraZk *handle_;
};

/// The library version, as reported by the C ABI.
inline std::string version() { return std::string(wickra_zk_version()); }

}  // namespace wickra_zk

#endif  // WICKRA_ZK_HPP
