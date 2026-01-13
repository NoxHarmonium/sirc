#!/usr/bin/env bash

# 16x16 tiles to test slicing
magick -size 512x512 xc:black \
  -channel R -fx "floor(i/32)/15" \
  -channel G -fx "floor(j/32)/15" \
  -channel B -fx "mod(floor(i/32)+floor(j/32),16)/15" \
  +channel tiles_16x16_512.png

magick -size 512x512 xc:black \
  -channel R -fx "mod(floor(i/32)+floor(j/32),2)" \
  -channel G -fx "mod(floor(i/32)+floor(j/32),2)" \
  -channel B -fx "mod(floor(i/32)+floor(j/32),2)" \
  +channel checker_16x16_512.png
