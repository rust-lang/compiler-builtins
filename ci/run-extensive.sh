#!/bin/bash

set -euo pipefail

echo "Tests to run: '$TO_TEST'"

if [ -z "$TO_TEST" ]; then
    echo "No tests to run, exiting."
    exit
fi

set -x

test_cmd=(
    cargo test
    --package libm-test
    --no-default-features
    # Test the generic implementations without `arch`. Most architecture
    # implementations are single instructions or short assembly sequences;
    # multi-step algorithms are tested separately below.
    #
    # libm/unstable enables libm/unstable-intrinsics, which means we usually
    # get the single-instruction ops anyway when we aren't specifically
    # testing for them.
    --features "libm-test/build-mpfr libm-test/unstable-float libm/unstable"
    --profile release-checked
)

# Run the non-extensive tests first to catch any easy failures
"${test_cmd[@]}" -- "$TO_TEST"

LIBM_EXTENSIVE_TESTS="$TO_TEST" "${test_cmd[@]}" -- extensive

# The table-based pow algorithm needs the same MPFR coverage as the generic one.
case ",$TO_TEST," in
    *,pow,*)
        "${test_cmd[@]}" --features libm/arch -- pow
        LIBM_EXTENSIVE_TESTS=pow "${test_cmd[@]}" --features libm/arch -- extensive
        ;;
esac
