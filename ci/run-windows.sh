#!/bin/bash

set -eux

target="$1"

m4 --help
make --help

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- \
    --default-toolchain nightly-x86_64-gnu \
    --target "$target" -y

cargo -vV

# rustup update nightly-x86_64-gnu
# rustup default nightly-x86_64-gnu
# rustup target add "$target"

./ci/run.sh "$target"
