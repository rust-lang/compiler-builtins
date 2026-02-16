#!/bin/bash

set -eux

export RUST_BACKTRACE="${RUST_BACKTRACE:-full}"
export NEXTEST_STATUS_LEVEL=all

target="${1:-}"

if [ -z "$target" ]; then
    host_target=$(rustc -vV | awk '/^host/ { print $2 }')
    echo "Defaulted to host target $host_target"
    target="$host_target"
fi

if [ "${USING_CONTAINER_RUSTC:-}" = 1 ]; then
    # Install nonstandard components if we have control of the environment
    rustup target list --installed |
        grep -E "^$target\$" ||
        rustup target add "$target"
fi

# If nextest is available, use that for tests
command -v cargo-nextest && nextest=1 || nextest=0
if [ "$nextest" = "1" ]; then
    test_cmd=(cargo nextest run --max-fail=20)

    # Workaround for https://github.com/nextest-rs/nextest/issues/2066
    if [ -f /.dockerenv ]; then
        cfg_file="/tmp/nextest-config.toml"
        echo "[store]" >> "$cfg_file"
        echo "dir = \"$CARGO_TARGET_DIR/nextest\"" >> "$cfg_file"
        test_cmd+=(--config-file "$cfg_file")
    fi

    # Not all configurations have tests to run on wasm
    [[ "$target" = *"wasm"* ]] && test_cmd+=(--no-tests=warn)

    profile_flag="--cargo-profile"
else
    test_cmd=(cargo test --no-fail-fast)
    profile_flag="--profile"
fi

# Cargo args for testing with `--workspace`
ws_flags=()

# We enumerate features manually.
ws_flags+=(--no-default-features)

# Enable arch-specific routines when available.
ws_flags+=(--features libm/arch)

# Always enable `unstable-float` since it expands available API but does not
# change any implementations.
ws_flags+=(--features unstable-float)

# We need to specifically skip tests for musl-math-sys on systems that can't
# build musl since otherwise `--all` will activate it.
case "$target" in
    # Can't build at all on MSVC, WASM, or thumb
    *windows-msvc*) ws_flags+=(--exclude musl-math-sys) ;;
    *wasm*) ws_flags+=(--exclude musl-math-sys) ;;
    *thumb*) ws_flags+=(--exclude musl-math-sys) ;;

    # We can build musl on MinGW but running tests gets a stack overflow
    *windows-gnu*) ;;
    # FIXME(#309): LE PPC crashes calling the musl version of some functions. It
    # seems like a qemu bug but should be investigated further at some point.
    # See <https://github.com/rust-lang/libm/issues/309>.
    *powerpc64le*) ;;

    # Everything else gets musl enabled
    *) ws_flags+=(--features libm-test/build-musl) ;;
esac

# Configure which targets test against MPFR
case "$target" in
    # MSVC cannot link MPFR
    *windows-msvc*) ;;
    # FIXME: MinGW should be able to build MPFR, but setup in CI is nontrivial.
    *windows-gnu*) ;;
    # Targets that aren't cross compiled in CI work fine
    aarch64*apple*) ws_flags+=(--features libm-test/build-mpfr) ;;
    aarch64*linux*) ws_flags+=(--features libm-test/build-mpfr) ;;
    i586*) ws_flags+=(--features libm-test/build-mpfr --features gmp-mpfr-sys/force-cross) ;;
    i686*) ws_flags+=(--features libm-test/build-mpfr) ;;
    x86_64*) ws_flags+=(--features libm-test/build-mpfr) ;;
esac

# FIXME: `STATUS_DLL_NOT_FOUND` testing macros on CI.
# <https://github.com/rust-lang/rust/issues/128944>
case "$target" in
    *windows-gnu) ws_flags+=(--exclude libm-macros) ;;
esac

# Test our implementation
if [ "${BUILD_ONLY:-}" = "1" ]; then
    # If we are on targets that can't run tests, verify that we can build.
    cmd=(cargo build --target "$target" --package compiler_builtins --package libm)
    "${cmd[@]}"
    "${cmd[@]}" --features unstable-intrinsics

    echo "can't run tests on $target; skipping"
else
    test_builtins=("${test_cmd[@]}" --target "$target" --package builtins-test)
    "${test_builtins[@]}"
    "${test_builtins[@]}" --release
    "${test_builtins[@]}" --features c
    "${test_builtins[@]}" --features c --release
    "${test_builtins[@]}" --features no-asm
    "${test_builtins[@]}" --features no-asm --release
    "${test_builtins[@]}" --benches
    "${test_builtins[@]}" --benches --release

    # Validate that having a verbatim path for the target directory works
    # (trivial to regress using `/` in paths to build artifacts rather than
    # `Path::join`). MinGW does not currently support these paths.
    if [[ "$target" = *"windows"* ]] && [[ "$target" != *"gnu"* ]]; then
        verb_path=$(cmd.exe //C echo \\\\?\\%cd%\\builtins-test\\target2)
        "${test_builtins[@]}" --target-dir "$verb_path" --features c
    fi

    # symcheck tests need specific env setup so it gets checked separately
    ws_flags+=(--workspace --exclude symbol-check --target "$target")

    if [ "${MAY_SKIP_LIBM_CI:-}" = "true" ]; then
        echo "skipping full libm PR CI"
        ws_flags+=(--exclude libm-test)
    fi

    test_ws=("${test_cmd[@]}" "${ws_flags[@]}")

    # Test once without any special features
    "${test_ws[@]}"

    # Run doctests if they were excluded by nextest
    [ "$nextest" = "1" ] && cargo test --doc --exclude compiler_builtins "${ws_flags[@]}"

    # Exclude the macros and utile crates from the rest of the tests to save CI
    # runtime, they shouldn't have anything feature- or opt-level-dependent.
    test_ws+=(
        --package libm --package libm-test
        --package compiler_builtins --package builtins-test
    )

    # Test once with intrinsics enabled
    "${test_ws[@]}" --features unstable-intrinsics
    "${test_ws[@]}" --features unstable-intrinsics --benches

    # Test the same in release mode, which also increases coverage. Also ensure
    # the soft float routines are checked.
    "${test_ws[@]}" "$profile_flag" release-checked
    "${test_ws[@]}" "$profile_flag" release-checked --features force-soft-floats
    "${test_ws[@]}" "$profile_flag" release-checked --features unstable-intrinsics
    "${test_ws[@]}" "$profile_flag" release-checked --features unstable-intrinsics --benches
fi

# Ensure there are no duplicate symbols or references to `core` when
# `compiler-builtins` is built with various features. Symcheck invokes Cargo to
# build with the arguments we provide it, then validates the built artifacts.
SYMCHECK_TEST_TARGET="$target" cargo test -p symbol-check --release
symcheck=(cargo run -p symbol-check --release)
symcheck+=(-- --build-and-check --target "$target")

# Executable section checks are meaningless on no-std targets
[[ "$target" == *"-none"* ]] && symcheck+=(--no-os)

"${symcheck[@]}" -- -p compiler_builtins
"${symcheck[@]}" -- -p compiler_builtins --release
"${symcheck[@]}" -- -p compiler_builtins --features c
"${symcheck[@]}" -- -p compiler_builtins --features c --release
"${symcheck[@]}" -- -p compiler_builtins --features no-asm
"${symcheck[@]}" -- -p compiler_builtins --features no-asm --release

run_intrinsics_test() {
    build_args=(--verbose --manifest-path builtins-test-intrinsics/Cargo.toml)
    build_args+=("$@")

    # symcheck also checks the results of builtins-test-intrinsics
    "${symcheck[@]}" -- "${build_args[@]}"

    # FIXME: we get access violations on Windows, our entrypoint may need to
    # be tweaked.
    if [ "${BUILD_ONLY:-}" != "1" ] && ! [[ "$target" = *"windows"* ]]; then
        cargo run --target "$target" "${build_args[@]}"
    fi
}

# Verify that we haven't dropped any intrinsics/symbols
run_intrinsics_test
run_intrinsics_test --release
run_intrinsics_test --features c
run_intrinsics_test --features c --release

# Verify that there are no undefined symbols to `panic` within our
# implementations
CARGO_PROFILE_DEV_LTO=true run_intrinsics_test
CARGO_PROFILE_RELEASE_LTO=true run_intrinsics_test --release

# Test libm

# Make sure a simple build works
cargo check -p libm --no-default-features --target "$target"

# Ensure that the routines do not panic.
#
# `--tests` must be passed because no-panic is only enabled as a dev
# dependency. The `release-opt` profile must be used to enable LTO and a
# single CGU.
ENSURE_NO_PANIC=1 cargo build \
    --package libm \
    --target "$target" \
    --no-default-features \
    --features unstable-float \
    --tests \
    --profile release-opt
