#!/bin/sh
# This script isn't enabled or run by default.
# If you'd like to use this hook for your own development,
# create a script named .git/hooks/pre-push that runs it

set -e

cargo test

# It looks like fuzzers occasionally break in nightly,
# which means maybe you should remove this line?
cargo +nightly fuzz build
