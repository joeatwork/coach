#!/bin/sh
# This script isn't enabled or run by default.
# If you'd like to use this hook for your own development,
# create a script named .git/hooks/pre-push that runs it

set -e

cargo test

# this doesn't run fuzzers, but does make sure
# that we haven't broken them
# Looks like +nightly has broken these for us :(
# cargo +nightly fuzz build
