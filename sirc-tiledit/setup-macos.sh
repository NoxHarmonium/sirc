#!/bin/bash

# Sets up a meson build directory on MacOS that uses the llvm toolchain installed
# by homebrew, instead of the xcode toolchain.
# It might be useful if you're having issues with missing headers in clang-tidy

# http://redsymbol.net/articles/unofficial-bash-strict-mode/
set -euo pipefail
IFS=$'\n\t'

SDK_PATH=$(xcrun --show-sdk-path)
CPU_FAMILY=$(uname -m | grep -q arm64 && echo 'aarch64' || echo 'x86_64')

sed "s|@SDK_PATH@|$SDK_PATH|g" native-macos-llvm.ini |
sed "s|cpu_family = .*|cpu_family = '$CPU_FAMILY'|" > /tmp/meson_macos.ini

meson setup build --native-file=/tmp/meson_macos.ini "$@"
