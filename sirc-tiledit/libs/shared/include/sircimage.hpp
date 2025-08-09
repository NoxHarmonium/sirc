#ifndef IMAGEPROCESSOR_H
#define IMAGEPROCESSOR_H

#include <array>
#include <cstdint>
#include <stdexcept>
#include <vector>

#include "constants.hpp"

enum class ImageChannel : std::uint8_t { R, G, B };

// The number of palette slots in the SIRC PPU
constexpr int MAX_PALETTE_SIZE = 256;

// SIRC uses a packed 16 bit color RGB (5bpp)
constexpr unsigned int SIRC_COLOR_COMPONENT_BITS = 5;
constexpr unsigned int SIRC_COLOR_RANGE = (1 << SIRC_COLOR_COMPONENT_BITS) - 1;

using SircColor = uint16_t;
using SircColorComponent = uint8_t;
using SircPalette = std::shared_ptr<std::vector<SircColor>>;
using PaletteReference = size_t;
using PackedSircPixelData =
    std::array<std::array<SircColor, HEIGHT_PIXELS>, WIDTH_PIXELS>;
using IndexedPixelData = std::array<PaletteReference, TOTAL_PIXELS>;

/**
 * @brief Represents an image in the format supported by the SIRC PPU
 *
 * The SIRC PPU uses a 15 bit (5bpp) color format with a palette.
 * The palette can store 256 colors but usually tile data will
 * only support max 4bpp (16 colors).
 */
struct SircImage {
  SircPalette palette;
  IndexedPixelData pixelData{};
  bool operator==(const SircImage &) const = default;

  static SircImage empty() {
    return {
        .palette = std::make_shared<std::vector<SircColor>>(),
        .pixelData = {},
    };
  };
};

inline SircColorComponent componentFromColor(const SircColor sircColor,
                                             const ImageChannel channel) {
  switch (channel) {
  case ImageChannel::R:
    return sircColor >> SIRC_COLOR_COMPONENT_BITS * 2 & SIRC_COLOR_RANGE;
  case ImageChannel::G:
    return sircColor >> SIRC_COLOR_COMPONENT_BITS & SIRC_COLOR_RANGE;
  case ImageChannel::B:
    return sircColor & SIRC_COLOR_RANGE;
  }
  throw std::runtime_error("Invalid ImageChannel value");
}

inline SircColor colorFromComponent(const SircColorComponent component,
                                    const ImageChannel channel) {
  switch (channel) {
  case ImageChannel::R:
    return component << SIRC_COLOR_COMPONENT_BITS * 2;
  case ImageChannel::G:
    return component << SIRC_COLOR_COMPONENT_BITS;
  case ImageChannel::B:
    return component;
  }
  throw std::runtime_error("Invalid ImageChannel value");
}

#endif // IMAGEPROCESSOR_H
