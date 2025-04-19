#[unsafe(no_mangle)]
#[linkage = "weak"]
pub unsafe extern "C" fn abort() -> ! {
    // On AVRs, an architecture that doesn't support traps, unreachable code
    // paths get lowered into calls to `abort`:
    //
    // https://github.com/llvm/llvm-project/blob/cbe8f3ad7621e402b050e768f400ff0d19c3aedd/llvm/lib/CodeGen/SelectionDAG/LegalizeDAG.cpp#L4462
    //
    // When control gets here, it means an undefined behavior has occurred, so
    // there's really not that much we can do to recover - we can't reliably
    // `panic!()`, because for all we know the environment is gone, so panicking
    // might end up with us getting back to this very function.
    //
    // So let's do the next best thing, loop.
    //
    // Alternatively we could (try to) restart the program, but since undefined
    // behavior is undefined, there's really no obligation for us to do anything
    // here - for all we care, we could just set the chip on fire; but that'd be
    // bad for the environment.

    loop {}
}
