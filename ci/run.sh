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

# Test our implementation
if [ "${BUILD_ONLY:-}" = "1" ]; then
    echo "nothing to do for no_std"
else
    run="cargo test --package builtins-test --no-fail-fast --target $target"
    $run
    $run --release
    $run --features c
    $run --features c --release
    $run --features no-asm
    $run --features no-asm --release
    $run --features no-f16-f128
    $run --features no-f16-f128 --release
    $run --benches
    $run --benches --release
fi

if [ "${TEST_VERBATIM:-}" = "1" ]; then
    verb_path=$(cmd.exe //C echo \\\\?\\%cd%\\builtins-test\\target2)
    cargo build --package builtins-test \
        --target "$target" --target-dir "$verb_path" --features c
fi

declare -a rlib_paths

# Set the `rlib_paths` global array to a list of all compiler-builtins rlibs
update_rlib_paths() {
    if [ -d /builtins-target ]; then
        rlib_paths=( /builtins-target/"${target}"/debug/deps/libcompiler_builtins-*.rlib )
    else
        rlib_paths=( target/"${target}"/debug/deps/libcompiler_builtins-*.rlib )
    fi
}

run_with_rlibs() {
    if [ -d /builtins-target ]; then
        rlib_paths=( /builtins-target/"${target}"/debug/deps/libcompiler_builtins-*.rlib )
    else
        rlib_paths=( target/"${target}"/debug/deps/libcompiler_builtins-*.rlib )
    fi

    "$@" "${rlib_paths[@]}"
}

# Remove any existing artifacts from previous tests that don't set #![compiler_builtins]
# update_rlib_paths
# rm -f "${rlib_paths[@]}"
run_with_rlibs rm -f

cargo build --target "$target"
cargo build --target "$target" --release
cargo build --target "$target" --features c
cargo build --target "$target" --release --features c
cargo build --target "$target" --features no-asm
cargo build --target "$target" --release --features no-asm
cargo build --target "$target" --features no-f16-f128
cargo build --target "$target" --release --features no-f16-f128

# Look out for duplicated symbols when we include the compiler-rt (C) implementation
run_with_rlibs cargo run -p symbol-check -- check-duplicates
run_with_rlibs rm -f

build_intrinsics_test() {
    cargo build --target "$target" -v --package builtins-test-intrinsics "$@"
}

# Verify that we haven't dropped any intrinsics/symbols
build_intrinsics_test
build_intrinsics_test --release
build_intrinsics_test --features c
build_intrinsics_test --features c --release

# Verify that there are no undefined symbols to `panic` within our
# implementations
CARGO_PROFILE_DEV_LTO=true \
    cargo build --target "$target" --package builtins-test-intrinsics
CARGO_PROFILE_RELEASE_LTO=true \
    cargo build --target "$target" --package builtins-test-intrinsics --release

# Ensure no references to any symbols from core
run_with_rlibs cargo run -p symbol-check -- check-core-syms
