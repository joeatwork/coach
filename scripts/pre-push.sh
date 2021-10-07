#!/bin/sh

set -e

cargo test

# this doesn't run fuzzers, but does make sure
# that we haven't broken them
cargo +nightly fuzz build
