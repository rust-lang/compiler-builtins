# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.12](https://github.com/rust-lang/compiler-builtins/compare/libm-v0.2.11...libm-v0.2.12) - 2025-04-20

### Other

- Run `cargo fmt` on all projects
- Add a .rustfmt.toml with style edition 2024
- Flatten the `libm/libm` directory
- Reorganize into compiler-builtins
- Remove compiler-builtins-smoke-test
- Update .git-blame-ignore-revs after the libm merge
- Update submodules after the `libm` merge
- Migrate all crates except `libm` to edition 2024
- Introduce a virtual manifest
- Move the `libm` crate to a subdirectory
- Mark generic functions `#[inline]`
- Combine the source files for `fmod`
- Ensure all public functions are marked `no_panic`
- Account for `PR_NUMBER` being set to an empty string
- Ensure configure.rs changes trigger rebuilds
- Increase the timeout for extensive tests
- Require `ci: allow-many-extensive` if a threshold is exceeded
- Allow skipping extensive tests with `ci: skip-extensive`
- Cancel jobs when a new push happens
- Combine the source files for more generic implementations
- Make `assert_biteq!` not rely on having `Int` in scope
- Add `NEG_NAN` to `Float`
- Correct the normalization of subnormals
- Add regression tests for subnormal issue
- Implement rounding for the hex float parsing and prepare to improve error handling
- Add assembly version of simple operations on aarch64
- Resolve small errors identified by recent clippy
- Replace calls to `core::arch` intrinsics with assembly
- Upgrade all dependencies to the latest
- Gate another assertion behind `compiler-builtins`
- Configure out remaining formatting when `compiler-builtins` is set
- Ignore unused variables when `compiler-builtins` is set
- Resolve monomorphization errors in `compiler-builtins`
- Make the compiler-builtins test more accurately mirror compiler-builtins
- Pin the nightly toolchain for aarch64 jobs
- Use `git ls-files` rather than manually globbing for tidy
- Make `fma` a trait method on `Float`
- fma refactor 3/3: combine `fma` public API with its implementation
- fma refactor 2/3: move math/generic/fma.rs to math/fma.rs
- fma refactor 1/3: remove math/fma.rs
- Scale test iteration count at a later point
- Add a way to print inputs on failure
- Rename `Float::exp` to `Float::ex`
- Check exact values for specified cases
- Add `roundeven{,f,f16,f128}`
- Fix parsing of negative hex float literals in util
- Increase allowed offset from infinity for ynf
- Add `fminimum`, `fmaximum`, `fminimum_num`, and `fmaximum_num`
- Combine `fmin{,f,f16,f128}` and `fmax{,f,f16,128}` into a single file
- Small refactor of bigint tests
- Eliminate the use of `force_eval!` in `ceil`, `floor`, and `trunc`
- Migrate away from nonfunctional `fenv` stubs
- Introduce a trait constant for the minimum positive normal value
- Implement `u256` with two `u128`s rather than `u64`
- Pin the nightly toolchain for i686-pc-windows-gnu
- Increase the tolerance for `jn` and `yn`
- Replace an `assert!` with `debug_assert!` in `u256::shr`
- Change how operators are `black_box`ed
- Add simple icount benchmarks for `u256` operations
- Decrease the allowed error for `cbrt`
- Port the CORE-MATH version of `cbrt`
- Add an enum representation of rounding mode
- Work arouind iai-callgrind apt failures
- Uncomment some hex float tests that should work now
- Convert `fmaf` to a generic implementation
- Remove or reduce the scope of `allow(unused)` where possible
- fix exponent calculation for subnormals
- Check more subnormal values during edge cases tests
- Run standard tests before running integration tests
- Add better edge case testing for `scalbn`
- Add `fmaf128`
- Make it possible to use `hf32!` and similar macros outside of `libm`
- Improve tidy output
- Add an integration test that verifies a list of cases
- Switch `musl` to track `master`
- Ensure zero has the correct sign
- Print the hex float format upon failure
- Commonize the signature for all instances of `get_test_cases`
- Start converting `fma` to a generic function
- Add checks via annotation that lists are sorted or exhaustive
- Do not add `libm_helper.rs` to the sources list
- Add a check in the `shared.rs` that the function list is sorted
- Add `scalbnf16`, `scalbnf128`, `ldexpf16`, and `ldexpf128`
- Fix hex float trait recursion problem
- Rename `EXP_MAX` to `EXP_SAT`
- Specify license as just MIT
- also print the hex float format for outputs
- Introduce a wrapper type for IEEE hex float formatting
- Support parsing NaN and infinities from the `hf*` functions
- Revert "Temporarily pin `indicatif` to 0.17.9"
- Temporarily pin `indicatif` to 0.17.9
- Switch musl from a script download to a submodule
- Ignore specific `atan2` and `sin` tests on i586
- Rework the available Cargo profiles
- Remove remnants of the `checked` feature
- Use `remquo` from Rug
- Use `frexp` from Rug
- Use `az` exported from Rug
- Upgrade all dependencies to the latest version
- Enable missing icount benchmarks
- Add `fmodf128`
- Add way to override the number of iterations for specific tests
- Increase or set CI timeouts
- Add `fmodf16` using the generic implementation
- Add a generic version of `fmod`
- Add `fminf16`, `fmaxf16`, `fminf128`, and `fmaxf128`
- Add a generic version of `fmin` and `fmax`
- Remove an outdated note about precision
- Add `roundf16` and `roundf128`
- Add a generic version of `round`
- Add a generic version of `scalbn`
- Change `from_parts` to take a `u32` exponent rather than `i32`
- Introduce XFAILs that assert failure
- Add `hf16!` and `hf128!`
- Fix the parsing of three-item tuples in `util`
- Add the ability to parse hex, binary, and float hex with util
- Add `rintf16` and `rintf128`
- Add a generic version of `rint`
- Adjust `ceil` style to be more similar to `floor`
- Add `floorf16` and `floorf128`
- Add a generic version of `floor`
- Add `ceilf16` and `ceilf128`
- Add a generic version of `ceil`
- Make `Float::exp` return an unsigned integer
- Shift then mask, rather than mask then shift
- Add `sqrtf16` and `sqrtf128`
- Copy the u256 implementation from compiler_builtins
- Port the most recent version of Musl's `sqrt` as a generic algorithm
- Enable `force-soft-floats` for extensive tests
- Don't set `opt_level` in the musl build script
- Add a retry to the musl download
- Remove trailing whitespace in scripts, run JuliaFormatter
- Ignore files relevant to benchmarking
- Add a way to ignore benchmark regression checks
- Run wall time benchmarks with `--features force-soft-floats`
- Run icount benchmarks once with softfloat and once with hardfloat
- Switch to the arm-linux runner and enable MPFR
- Remove the limit for querying a baseline
- Add an xfail for recent ynf failures
- Reduce the warm up and measurement time for `short-benchmarks`
- Run iai-callgrind benchmarks in CI
- Add benchmarks using iai-callgrind
- Provide a way to override iteration count
- Increase the CI timeout
- Adjust precision and add xfails based on new tests
- Replace `HasDomain` to enable multi-argument edge case and domain tests
- Add an override for a recent failure
- Pass --max-fail to nextest so it doesn't fail fast
- Slightly restructure `ci/calculate-exhaustive-matrix.py`
- Change `.yml` files to the canonical extension `.yaml`
- Use cargo-nextest for running tests in CI
- Simplify and optimize `fdim` ([#442](https://github.com/rust-lang/compiler-builtins/pull/442))
- Reduce indentation in `run.sh` using early return
- Don't set `codegen-units=1` by default in CI
- Add `fdimf16` and `fdimf128`
- Add a generic version of `fdim`
- Format the MPFR manual implementation list
- Disable `util` and `libm-macros` for optimized tests
- Add `truncf16` and `truncf128`
- Add a generic version of `trunc`
- Add a utility crate for quick evaluation
- Enable `build-mpfr` and `build-musl` by default
- Rename the `test-multiprecision` feature to `build-mpfr`
- Introduce arch::aarch64 and use it for rint{,f}
- Use wasm32 arch intrinsics for rint{,f}
- Add a new precision adjustment for i586 `exp2f`
- Add a new precision adjustment for i586 `rint`
- Expose C versions of `libm` functions in the `cb` crate
- Always use the same seed for benchmarking
- Add `biteq` and `exp_unbiased` to `Float`
- Add a `release-checked` profile with debug and overflow assertions
- Remove `ExpInt` from `Float`, always use `i32` instead
- Reorder tests in `run.sh`
- Split `cast` into `cast` and `cast_lossy`
- Use `core::arch::wasm` functions rather than intrinsics
- Add tests against MPFR for `remquo` and `remquof`
- Account for optimization levels other than numbers
- Make extensive tests exhaustive if there are enough iterations available
- Increase the allowed ULP for `tgammaf`
- Replace "intrinsic" config with "arch" config
- Don't use intrinsics abs for `f16` and `f128` on wasm32
- Remove an unused `feature = "force-soft-floats"` gate
- Switch from using `unstable-intrinsics` to `intrinsics_enabled`
- Increase the allowed precision for failing tests on i586
- Enable MPFR tests on i586
- Only update the github ref for pull requests
- Loosen precision on i586 based on new tests
- Add an override for failing ceil/floor tests on i586
- Add domain and edge case tests to musl
- Add test infrastructure for `f16` and `f128`
- Add `fabsf16`, `fabsf128`, `copysignf16`, and `copysignf128`
- Enable `f16` and `f128` when creating the API change list
- Run extensive tests in CI when relevant files change
- Update precision based on failures from extensive tests
- Add extensive and exhaustive tests
- Add more detailed definition output for `update-api-list.py`
- Add tests against MPFR for `ilogb` and `ilogbf`
- Increase the precision for `jn` and `jnf`
- Rename `unstable-test-support` to `unstable-public-internals`
- Update precision based on new test results
- Rewrite the random test generator
- Add an iterator that ensures known size
- Streamline the way that test iteration count is determined
- Add a way for tests to log to a file
- Add tests against MPFR for `scalbn{f}` and `ldexp{f}`
- Add tests against MPFR for `frexp` and `frexpf`
- Add tests against MPFR for `modf` and `modff`
- Clean up integers stored in `MpTy`
- Sort `ilogb` with other precise operations
- Change to exhaustive matching for `default_ulp`
- Use intrinsics for `abs` and `copysign` when available
- Rename generic `abs` to `fabs`
- Always emit `f16_enabled` and `f128_enabled` attributes
- Add missing functions to the macro list
- Use `rustdoc` output to create a list of public API
- Forward the `CI` environment variable when running in Docker
- Remove lossy casting in `logspace`
- Set the allowed FMA ULP to 0
- Don't run `push` CI on anything other than `master`
- Use `CheckCtx` in more places
- Move `CheckBasis` and `CheckCtx` to a new `run_cfg` module
- Add `ALL`, `from_str` and `math_op` to `Identifier`
- Add new trait implementations for `Identifier` and `BaseName`
- Include `shared.rs` in `libm_test::op`
- Move the macro's input function list to a new module `shared`
- Add a way to plot the output from generators
- Update allowed precision to account for new tests
- Add tests for edge cases
- Add interfaces and tests based on function domains
- Introduce a float extension trait and some numerical routines
- Add an 8-bit float type for testing purposes
- Remove an `is_nan` workaround that is no longer needed
- Update and slightly refactor some of the `Float` trait
- Always enable `unstable-float` in CI
- Add `f16` and `f128` configuration from `compiler-builtins`
- Introduce generic `abs` and `copysign`
- Change from `-latest` to named CI images
- Allow Clippy lints in `compiler-builtins-smoke-test`
- Fix new `clippy::precedence` lints
- Replace string function name matching with enums where possible
- Rename associated type helpers, add `OpITy`
- Introduce helper types for accessing trait items
- Fix a bug in `abs_diff`
- Remove tests against system musl
- Use `https:` links in `README.md`
- Move some numeric trait logic to default implementations
- Change the `multiprec_` prefix to `mp_`
- Change default ULP to use enum matching
- Rename `Name` to `Identifier` to avoid some ambiguity of "name"
- Change the `CheckCtx` constructor to take a `Name` enum
- Correct the proc macro to emit `pub` functions
- Rework tests to make use of the new `MathOp` trait
- Introduce a `op` module with struct representations of each routine
- Adjust how the proc macro emits types and add an enum
- Fix clippy lints in `crates/` and enable this on CI
- Resolve clippy errors in `libm` tests and check this in CI
- Add some more basic docstrings ([#352](https://github.com/rust-lang/compiler-builtins/pull/352))
- Introduce `hf32!` and `hf64!` macros for hex float support
- Enable clippy for `libm` in CI
- Fix errors reported by Clippy in `libm`
- Replace `libm_test::{Float, Int}` with `libm::{Float, Int}`
- Expose the `support` module publicly with a test feature
- Update libm `Float` and `Int` with functions from the test traits
- Change prefixes used by the `Float` trait
- Check benchmarks in CI
- Remove `libm-bench`
- Add benchmarks against musl libm
- add support for loongarch64-unknown-linux-gnu
- Rename `canonical_name` to `base_name`
- Add float and integer traits from compiler-builtins
- Move architecture-specific code to `src/math/arch`
- Update `select_implementation` to accept arch configuration
- Add an "arch" Cargo feature that is on by default
- Vendor `cfg_if::cfg_if!`
- Update `libm-test/build.rs` to skip directories
- Rename the `special_case` module to `precision` and move default ULP
- Run tests against MPFR on CI where possible
- Add a test against MPFR using random inputs
- Create interfaces for testing against MPFR
- Combine the WASM CI job with the others
- Make use of `select_implementation`
- Introduce a `select_implementation` macro
- Introduce `math::arch::intrinsics`
- Replace `feature = "unstable-intrinsics"` with `intrinsics_enabled`
- Move the existing "unstable" feature to "unstable-intrinsics"

## [0.2.11](https://github.com/rust-lang/libm/compare/libm-v0.2.10...libm-v0.2.11) - 2024-10-28

### Fixed

- fix type of constants in ported sincosf ([#331](https://github.com/rust-lang/libm/pull/331))

### Other

- Disable a unit test that is failing on i586
- Add a procedural macro for expanding all function signatures
- Introduce `musl-math-sys` for bindings to musl math symbols
- Add basic docstrings to some functions ([#337](https://github.com/rust-lang/libm/pull/337))

## [0.2.10](https://github.com/rust-lang/libm/compare/libm-v0.2.9...libm-v0.2.10) - 2024-10-28

### Other

- Set the MSRV to 1.63 and test this in CI

## [0.2.9](https://github.com/rust-lang/libm/compare/libm-v0.2.8...libm-v0.2.9) - 2024-10-26

### Fixed

- Update exponent calculations in nextafter to match musl

### Changed

- Update licensing to MIT AND (MIT OR Apache-2.0), as this is derivative from
  MIT-licensed musl.
- Set edition to 2021 for all crates
- Upgrade all dependencies

### Other

- Don't deny warnings in lib.rs
- Rename the `musl-bitwise-tests` feature to `test-musl-serialized`
- Rename the `musl-reference-tests` feature to `musl-bitwise-tests`
- Move `musl-reference-tests` to a new `libm-test` crate
- Add a `force-soft-floats` feature to prevent using any intrinsics or
  arch-specific code
- Deny warnings in CI
- Fix `clippy::deprecated_cfg_attr` on compiler_builtins
- Corrected English typos
- Remove unneeded `extern core` in `tgamma`
- Allow internal_features lint when building with "unstable"

## [v0.2.1] - 2019-11-22

### Fixed

- sincosf

## [v0.2.0] - 2019-10-18

### Added

- Benchmarks
- signum
- remainder
- remainderf
- nextafter
- nextafterf

### Fixed

- Rounding to negative zero
- Overflows in rem_pio2 and remquo
- Overflows in fma
- sincosf

### Removed

- F32Ext and F64Ext traits

## [v0.1.4] - 2019-06-12

### Fixed

- Restored compatibility with Rust 1.31.0

## [v0.1.3] - 2019-05-14

### Added

- minf
- fmin
- fmaxf
- fmax

## [v0.1.2] - 2018-07-18

### Added

- acosf
- asin
- asinf
- atan
- atan2
- atan2f
- atanf
- cos
- cosf
- cosh
- coshf
- exp2
- expm1
- expm1f
- expo2
- fmaf
- pow
- sin
- sinf
- sinh
- sinhf
- tan
- tanf
- tanh
- tanhf

## [v0.1.1] - 2018-07-14

### Added

- acos
- acosf
- asin
- asinf
- atanf
- cbrt
- cbrtf
- ceil
- ceilf
- cosf
- exp
- exp2
- exp2f
- expm1
- expm1f
- fdim
- fdimf
- floorf
- fma
- fmod
- log
- log2
- log10
- log10f
- log1p
- log1pf
- log2f
- roundf
- sinf
- tanf

## v0.1.0 - 2018-07-13

- Initial release

[Unreleased]: https://github.com/japaric/libm/compare/v0.2.1...HEAD
[v0.2.1]: https://github.com/japaric/libm/compare/0.2.0...v0.2.1
[v0.2.0]: https://github.com/japaric/libm/compare/0.1.4...v0.2.0
[v0.1.4]: https://github.com/japaric/libm/compare/0.1.3...v0.1.4
[v0.1.3]: https://github.com/japaric/libm/compare/v0.1.2...0.1.3
[v0.1.2]: https://github.com/japaric/libm/compare/v0.1.1...v0.1.2
[v0.1.1]: https://github.com/japaric/libm/compare/v0.1.0...v0.1.1
