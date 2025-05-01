#!/bin/bash

set -eux

target="$1"

m4 --help
make --help

rustup update nightly-x86_64-gnu
rustup default nightly-x86_64-gnu
rustup target add "$target"

./ci/run.sh ${{ matrix.target }}
