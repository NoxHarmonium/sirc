#!/bin/bash

# Sets up a meson build directory on MacOS that uses the llvm toolchain installed
# by homebrew, instead of the xcode toolchain.
# It might be useful if you're having issues with missing headers in clang-tidy

# http://redsymbol.net/articles/unofficial-bash-strict-mode/
set -euo pipefail
IFS=$'\n\t'

meson setup build --native-file=./native-macos-llvm.ini "$@"
