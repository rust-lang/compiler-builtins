#!/bin/bash
# Install needed dependencies for gungraun.

set -eux

target="${1:-}"

# Needed for gungraun
to_install=(valgrind gdb libc6-dbg)

[[ "$target" = *"i686"* ]] && to_install+=(gcc-multilib)

# specific check for armv7-unknown-linux-gnueabihf on canonical runners which
# are 32-bit armhf-native but have VMs running over an arm64 kernel with
# AArch32/EL0 support
if [ "$target" = "armv7-unknown-linux-gnueabihf" ] && [ "${RUN_IN_DOCKER:-}" != "true" ] &&
    [ "$(dpkg --print-architecture 2>/dev/null)" = "arm64" ]; then

    # expose these variables so we can keep the arm64 host toolchain, cross-link with
    # the armhf gcc, and run the test binaries natively in AArch32 mode
    {
        echo "CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc"
        echo "CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc"
        echo "AR_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-ar"
    } >> "$GITHUB_ENV"

    # install required tooling to enable cross-compilation
    sudo dpkg --add-architecture armhf
    to_install+=(gcc-arm-linux-gnueabihf) # cross-compiler
    to_install+=(libc6-dev-armhf-cross) # target libc
    to_install+=(libc6:armhf libgcc-s1:armhf) # loaded at runtime
fi

sudo apt-get update
sudo apt-get install -y "${to_install[@]}"

rustup update "$BENCHMARK_RUSTC" --no-self-update
rustup default "$BENCHMARK_RUSTC"
[ -n "$target" ] && rustup target add "$target"

# Install the version of gungraun-runner that is specified in Cargo.toml
gungraun_version="$(cargo metadata --format-version=1 --features icount |
    jq -r '.packages[] | select(.name == "gungraun").version')"

cargo_install=(cargo install)
command -v cargo-binstall && cargo_install=(cargo binstall -y)

"${cargo_install[@]}" gungraun-runner --version "$gungraun_version"
