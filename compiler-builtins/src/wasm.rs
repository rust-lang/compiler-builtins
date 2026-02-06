//! Builtins for WebAssembly targets.

// Define the __cpp_exception tag that LLVM's wasm exception handling requires.
// This must be provided since LLVM commit
// aee99e8015daa9f53ab1fd4e5b24cc4c694bdc4a which changed the tag from being
// weakly defined in each object file to being an external reference that must
// be linked from somewhere.
//
// In LLVM's compiler-rt this is provided by
// compiler-rt/lib/builtins/wasm/__cpp_exception.S
//
// Emscripten provides this tag in its runtime libraries, so we don't need to
// define it for Emscripten targets. Including this in Emscripten would break
// the wasm-cpp-exception-tag test.
#[cfg(not(target_os = "emscripten"))]
core::arch::global_asm!(
    ".globl __cpp_exception",
    #[cfg(target_pointer_width = "64")]
    ".tagtype __cpp_exception i64",
    #[cfg(target_pointer_width = "32")]
    ".tagtype __cpp_exception i32",
    "__cpp_exception:",
);
