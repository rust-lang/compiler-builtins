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

# # Test our implementation
# if [ "${BUILD_ONLY:-}" = "1" ]; then
#     echo "nothing to do for no_std"
# else
#     run="cargo test --package builtins-test --no-fail-fast --target $target"
#     $run
#     $run --release
#     $run --features c
#     $run --features c --release
#     $run --features no-asm
#     $run --features no-asm --release
#     $run --features no-f16-f128
#     $run --features no-f16-f128 --release
#     $run --benches
#     $run --benches --release
# fi

# if [ "${TEST_VERBATIM:-}" = "1" ]; then
#     verb_path=$(cmd.exe //C echo \\\\?\\%cd%\\builtins-test\\target2)
#     cargo build --package builtins-test \
#         --target "$target" --target-dir "$verb_path" --features c
# fi


## REMOVE
PREFIX=${target//unknown-/}-
case "$target" in
    armv7-*)
        PREFIX=arm-linux-gnueabihf-
        ;;
    thumb*)
        PREFIX=arm-none-eabi-
        ;;
    *86*-*)
        PREFIX=
        ;;
esac

NM=$(find "$(rustc --print sysroot)" \( -name llvm-nm -o -name llvm-nm.exe \) )
if [ "$NM" = "" ]; then
  NM="${PREFIX}nm"
fi

# i686-pc-windows-gnu tools have a dependency on some DLLs, so run it with
# rustup run to ensure that those are in PATH.
TOOLCHAIN="$(rustup show active-toolchain | sed 's/ (default)//')"
if [[ "$TOOLCHAIN" == *i686-pc-windows-gnu ]]; then
  NM="rustup run $TOOLCHAIN $NM"
fi

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

# for_each_rlib nm -A

declare -a rlib_paths

# Set the `rlib_paths` global array to a list of all compiler-builtins rlibs
update_rlib_paths() {
    if [ -d /builtins-target ]; then
        rlib_paths=( /builtins-target/"${target}"/debug/deps/libcompiler_builtins-*.rlib )
    else
        rlib_paths=( target/"${target}"/debug/deps/libcompiler_builtins-*.rlib )
    fi
}


update_rlib_paths
for rlib in "${rlib_paths[@]}"; do
    set +x
    echo "================================================================"
    echo "checking $rlib for references to core"
    echo "================================================================"
    set -x

    tmpdir="${CARGO_TARGET_DIR:-target}/tmp"
    test -d "$tmpdir" || mkdir "$tmpdir"
    defined="$tmpdir/defined_symbols.txt"
    undefined="$tmpdir/defined_symbols.txt"

    $NM --quiet -U "$rlib" | grep 'T _ZN4core' | awk '{print $3}' | sort | uniq > "$defined"
    $NM --quiet -u "$rlib" | grep 'U _ZN4core' | awk '{print $2}' | sort | uniq > "$undefined"
    grep_has_results=0
    grep -v -F -x -f "$defined" "$undefined" && grep_has_results=1

    wc -l $defined
    wc -l $undefined

    if [ "$target" = "powerpc64-unknown-linux-gnu" ]; then
        echo "FIXME: powerpc64 fails these tests"
    elif [ "$grep_has_results" != 0 ]; then
        echo "error: found unexpected references to core"
        exit 1
    else
        echo "success; no references to core found"
    fi
done
## END REMOVE


# Run a command for each `compiler_builtins` rlib file
for_each_rlib() {
    if [ -d /builtins-target ]; then
        rlib_paths=( /builtins-target/"${target}"/debug/deps/libcompiler_builtins-*.rlib )
    else
        rlib_paths=( target/"${target}"/debug/deps/libcompiler_builtins-*.rlib )
    fi

    if [ "${#rlib_paths[@]}" -lt 1 ]; then
        echo "rlibs expected but not found"
        exit 1
    fi

    "$@" "${rlib_paths[@]}"
}

# Remove any existing artifacts from previous tests that don't set #![compiler_builtins]
for_each_rlib rm -f

cargo build --target "$target"
cargo build --target "$target" --release
cargo build --target "$target" --features c
cargo build --target "$target" --release --features c
cargo build --target "$target" --features no-asm
cargo build --target "$target" --release --features no-asm
cargo build --target "$target" --features no-f16-f128
cargo build --target "$target" --release --features no-f16-f128

# The 
symcheck=(cargo run -p symbol-check)
[[ "$target" = "wasm"* ]] && symcheck+=(--features wasm)

# Look out for duplicated symbols when we include the compiler-rt (C) implementation
for_each_rlib "${symcheck[@]}" -- check-duplicates
for_each_rlib rm -f

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

for_each_rlib $NM -A

# Ensure no references to any symbols from core
for_each_rlib "${symcheck[@]}" -- check-core-syms


