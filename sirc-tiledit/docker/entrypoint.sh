#!/usr/bin/env bash

cd /project
meson setup /builder/build

cd /builder/build
"$@"