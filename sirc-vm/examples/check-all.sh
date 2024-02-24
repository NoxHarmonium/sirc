#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

# Used to run all the examples to smoke test changes
# Takes a while because it does a clean for each example to make sure there aren't any stale object files etc. sitting around

MAKEFILES="$(find . -mindepth 2 -maxdepth 2 -type f -name Makefile)";

for MAKEFILE in $MAKEFILES; do \
    make -C $(dirname "$MAKEFILE") clean check; \
done
