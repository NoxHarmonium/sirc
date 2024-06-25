#ifndef IMAGEPROCESSOR_H
#define IMAGEPROCESSOR_H

#include <array>
#include <cstddef>
#include <cstdint>
#include <map>
#include <vector>

constexpr int WIDTH_PIXELS = 256;
constexpr int HEIGHT_PIXELS = 256;
// The number of palette slots in the SIRC PPU
constexpr int MAX_PALETTE_SIZE = 256;

// SIRC uses a packed 16 bit color RGB (5bpp)
constexpr unsigned int SIRC_COLOR_COMPONENT_BITS = 5;
constexpr unsigned int SIRC_COLOR_RANGE =
    (1 << (SIRC_COLOR_COMPONENT_BITS)) - 1;

using SircColor = uint16_t;
using ArgbColor = uint32_t;
using PaletteReference = uint8_t;
using PackedPixelData =
    std::array<std::array<SircColor, HEIGHT_PIXELS>, WIDTH_PIXELS>;
using IndexedPixelData =
    std::array<std::array<PaletteReference, HEIGHT_PIXELS>, WIDTH_PIXELS>;

struct SircImageData {
  std::vector<SircColor> palette;
  IndexedPixelData pixelData;
  bool operator==(const SircImageData &) const = default;
};

/**
 * @brief Represents an image in the format supported by the SIRC PPU
 *
 * The SIRC PPU uses a 15 bit (5bpp) color format with a palette.
 * The palette can store 256 colors but usually tile data will
 * only support max 4bpp (16 colors).
 */
class SircImage {

public:
  static SircImage fromPixelData(const PackedPixelData &pixelData);
  static SircImage fromSircImageData(const SircImageData &imageData);

  [[nodiscard]] SircImageData getImageData() const;
  [[nodiscard]] PaletteReference paletteIndexForColor(SircColor color) const;

private:
  SircImageData imageData = {};
  // This isn't doing anything at the moment because we are part way through a
  // refactor
  std::map<SircColor, size_t> paletteLookup;

  SircImage();
};

#endif // IMAGEPROCESSOR_H
