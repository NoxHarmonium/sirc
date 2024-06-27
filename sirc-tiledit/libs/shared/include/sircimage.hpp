#ifndef IMAGEPROCESSOR_H
#define IMAGEPROCESSOR_H

#include <array>
#include <cstddef>
#include <cstdint>
#include <map>
#include <vector>

#include "constants.hpp"

// The number of palette slots in the SIRC PPU
constexpr int MAX_PALETTE_SIZE = 256;

// SIRC uses a packed 16 bit color RGB (5bpp)
constexpr unsigned int SIRC_COLOR_COMPONENT_BITS = 5;
constexpr unsigned int SIRC_COLOR_RANGE =
    (1 << (SIRC_COLOR_COMPONENT_BITS)) - 1;

using SircColor = uint16_t;
using PaletteReference = size_t;
using PackedSircPixelData =
    std::array<std::array<SircColor, HEIGHT_PIXELS>, WIDTH_PIXELS>;
using IndexedPixelData =
    std::array<std::array<PaletteReference, HEIGHT_PIXELS>, WIDTH_PIXELS>;

/**
 * @brief Represents an image in the format supported by the SIRC PPU
 *
 * The SIRC PPU uses a 15 bit (5bpp) color format with a palette.
 * The palette can store 256 colors but usually tile data will
 * only support max 4bpp (16 colors).
 */
struct SircImage {
  std::vector<SircColor> palette;
  IndexedPixelData pixelData;
  bool operator==(const SircImage &) const = default;
};

#endif // IMAGEPROCESSOR_H
