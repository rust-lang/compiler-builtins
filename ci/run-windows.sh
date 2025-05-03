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

echo 'export PATH="/c/Users/$USERNAME/.cargo/bin:$PATH"' >> ~/.bashrc

echo "$PATH"
