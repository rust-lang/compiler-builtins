#!/bin/bash

set -eux

target="$1"

m4 --help
make --help

echo "$PATH"

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- \
    --default-toolchain nightly-x86_64-gnu \
    --target "$target" \
    --profile minimal -y

# Wrapper to ease managerment of environments
pacman --noconfirm -S pactoys
# Install C toolchain according to `MSYSTEM`
pacboy --noconfirm -S toolchain

pacman --noconfirm -S mingw-w64-x86_64-toolchain

# maybe only need mingw-w64-x86_64-gcc?

ls ~/
echo 'export PATH="/c/Users/$USERNAME/.cargo/bin:$PATH"' >> ~/.bash_profile

echo "$PATH"
