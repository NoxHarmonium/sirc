#!/bin/bash
# http://redsymbol.net/articles/unofficial-bash-strict-mode/
set -euo pipefail
IFS=$'\n\t'

##
## Updates the expected images so they match the current output
##
## Useful when the output changes due to an improvement/bug fix and you need to make the tests pass
##

for actual_file in *_output_actual_*.png;
do
    expected_file="${actual_file/_output_actual_/_output_expected_}"
    cp "$actual_file" "$expected_file"
    echo "Copied $actual_file to $expected_file"
done