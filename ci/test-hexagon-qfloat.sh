#!/usr/bin/env bash
# Requires nightly Rust with rust-src, Cargo, Perl, and llvm-nm.
set -euo pipefail

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
manifest="$repo/Cargo.toml"
shim_manifest="$repo/builtins-shim/Cargo.toml"
target=hexagon-unknown-linux-musl
nm=${LLVM_NM:-llvm-nm}

symbols=(
    __qf_convert_hf_to_b_rne
    __qf_convert_hf_to_h_rne
    __qf_convert_hf_to_ub_rne
    __qf_convert_hf_to_uh_rne
    __qf_fast_convert_hf_to_b_rne
    __qf_fast_convert_hf_to_h_rne
    __qf_fast_convert_hf_to_ub_rne
    __qf_fast_convert_hf_to_uh_rne
)

check_target() {
    local cpu=$1 features=$2 expected=$3 archive actual
    # Select the local crate, not the compiler-builtins used to build core.
    archive=$(
        RUSTFLAGS="-C target-cpu=$cpu -C target-feature=$features" \
            cargo build --manifest-path "$manifest" -p compiler_builtins \
            --target "$target" -Z build-std=core --message-format=json |
            SHIM_MANIFEST="$shim_manifest" perl -MJSON::PP -ne '
                my $event = decode_json($_);
                if ($event->{reason} eq "compiler-artifact" &&
                    $event->{manifest_path} eq $ENV{SHIM_MANIFEST}) {
                    for my $file (@{$event->{filenames}}) {
                        print "$file\n" if $file =~ /\.rlib$/;
                    }
                }
            '
    )
    test -n "$archive"
    actual=$("$nm" --defined-only "$archive" | awk '$NF ~ /^__qf_(fast_)?convert_hf_to_(b|h|ub|uh)_rne$/ {print $NF}' | sort)
    if [[ "$expected" == present ]]; then
        diff -u <(printf '%s\n' "${symbols[@]}" | sort) <(printf '%s\n' "$actual")
    else
        test -z "$actual"
    fi
    printf '%s %s: QFloat symbols %s\n' "$cpu" "$features" "$expected"
}

check_target hexagonv79 +hvx,+hvx-length128b,+hvx-ieee-fp,+hvx-qfloat present
check_target hexagonv79 +hvx,+hvx-length64b,+hvx-ieee-fp,+hvx-qfloat present
check_target hexagonv79 -hvx,-hvx-ieee-fp,-hvx-qfloat absent
check_target hexagonv79 +hvx,+hvx-length128b,+hvx-ieee-fp,-hvx-qfloat absent
check_target hexagonv79 +hvx,+hvx-length128b,-hvx-ieee-fp,+hvx-qfloat absent
check_target hexagonv68 -hvx absent
