// Package wickra provides idiomatic Go bindings for wickra-zk over its C ABI
// hub: create a stateless Prover, drive it with command JSON (prove, verify,
// version) and read back the response JSON — the same protocol as
// the CLI and every other binding.
//
// The binding links the prebuilt C ABI library, staged per platform under
// ./lib/<goos>_<goarch>/, with the header vendored under ./include.
package wickra

/*
#cgo CFLAGS: -I${SRCDIR}/include
#cgo linux,amd64 LDFLAGS: -L${SRCDIR}/lib/linux_amd64 -lwickra_zk -Wl,-rpath,${SRCDIR}/lib/linux_amd64
#cgo linux,arm64 LDFLAGS: -L${SRCDIR}/lib/linux_arm64 -lwickra_zk -Wl,-rpath,${SRCDIR}/lib/linux_arm64
#cgo darwin,amd64 LDFLAGS: -L${SRCDIR}/lib/darwin_amd64 -lwickra_zk -Wl,-rpath,${SRCDIR}/lib/darwin_amd64
#cgo darwin,arm64 LDFLAGS: -L${SRCDIR}/lib/darwin_arm64 -lwickra_zk -Wl,-rpath,${SRCDIR}/lib/darwin_arm64
#cgo windows,amd64 LDFLAGS: -L${SRCDIR}/lib/windows_amd64 -l:wickra_zk.dll
#cgo windows,arm64 LDFLAGS: -L${SRCDIR}/lib/windows_arm64 -l:wickra_zk.dll
#include <stdlib.h>
#include "wickra_zk.h"
*/
import "C"

import (
	"fmt"
	"runtime"
	"unsafe"
)

// Prover is a stateless prover driven by JSON commands.
type Prover struct {
	handle *C.WickraZk
}

// New creates a stateless prover. Call Close when done (a finalizer also frees
// it, but explicit Close is preferred).
func New() *Prover {
	p := &Prover{handle: C.wickra_zk_new()}
	runtime.SetFinalizer(p, (*Prover).Close)
	return p
}

// Command applies a command JSON and returns the response JSON. It uses the C
// ABI's length-out protocol: a first call learns the length, then the response
// is read into a caller-owned buffer.
func (p *Prover) Command(cmdJSON string) (string, error) {
	ccmd := C.CString(cmdJSON)
	defer C.free(unsafe.Pointer(ccmd))

	n := C.wickra_zk_command(p.handle, ccmd, nil, 0)
	if n < 0 {
		return "", fmt.Errorf("wickra-zk: command failed (code %d)", int(n))
	}
	buf := make([]byte, int(n)+1)
	C.wickra_zk_command(
		p.handle,
		ccmd,
		(*C.char)(unsafe.Pointer(&buf[0])),
		C.size_t(len(buf)),
	)
	return string(buf[:n]), nil
}

// Close frees the prover handle. Safe to call more than once.
func (p *Prover) Close() {
	if p.handle != nil {
		C.wickra_zk_free(p.handle)
		p.handle = nil
	}
	runtime.SetFinalizer(p, nil)
}

// Version returns the library version.
func Version() string {
	return C.GoString(C.wickra_zk_version())
}
