
#include "rgbaadapter.hpp"
#include "sircimage.hpp"

#include <cassert>
#include <miscadapter.hpp>

SircColor sircColorFromRgba(const RgbaPixel rgbaColor) {
  const RgbaComponent r = rgbaColor >> 24 & 0xFF;
  const RgbaComponent g = rgbaColor >> 16 & 0xFF;
  const RgbaComponent b = rgbaColor >> 8 & 0xFF;
  const RgbaComponent a = rgbaColor & 0xFF;
  // Any pixel less than 100% alpha is ignored and counted as black
  if (a < RGBA_COMPONENT_MAX) {
    return 0x0;
  }
  // Thanks https://threadlocalmutex.com/?p=48 for fast dept conversion
  const SircColorComponent scaledR = (r * 249 + 1024) >> 11;
  const SircColorComponent scaledG = (g * 249 + 1024) >> 11;
  const SircColorComponent scaledB = (b * 249 + 1024) >> 11;

  return scaledR << SIRC_COLOR_COMPONENT_BITS * 2 |
         scaledG << SIRC_COLOR_COMPONENT_BITS | scaledB;
}

RgbaPixel rgbaFromSircColor(const SircColor sircColor) {
  const unsigned int sircR =
      sircColor >> SIRC_COLOR_COMPONENT_BITS * 2 & SIRC_COLOR_RANGE;
  const unsigned int sircG =
      sircColor >> SIRC_COLOR_COMPONENT_BITS & SIRC_COLOR_RANGE;
  const unsigned int sircB = sircColor & SIRC_COLOR_RANGE;

  return static_cast<RgbaPixel>(
             // Thanks https://threadlocalmutex.com/?p=48 for fast dept
             // conversion
             static_cast<RgbaComponent>((sircR * 527 + 23) >> 6) << 24 |
             static_cast<RgbaComponent>((sircG * 527 + 23) >> 6) << 16 |
             static_cast<RgbaComponent>((sircB * 527 + 23) >> 6) << 8) |
         // Alpha is always 100% for now
         RGBA_COMPONENT_MAX;
}

SircImage RgbaAdapter::rgbaToSircImage(const RgbaPixelData &pixelData) {
  PackedSircPixelData convertedPixelData;

  for (int x = 0; x < WIDTH_PIXELS; x++) {
    for (int y = 0; y < HEIGHT_PIXELS; y++) {
      // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
      const auto pixel = pixelData[x][y];
      const auto convertedPixel = sircColorFromRgba(pixel);
      // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
      convertedPixelData[x][y] = convertedPixel;
    }
  }
  auto sircImage =
      MiscAdapter::packedSircPixelDataToSircImage(convertedPixelData);

  return sircImage;
}

RgbaPixelData RgbaAdapter::sircImageToRgba(const SircImage &sircImage) {
  RgbaPixelData output;
  auto [palette, pixelData] = sircImage;

  for (int x = 0; x < WIDTH_PIXELS; x++) {
    for (int y = 0; y < HEIGHT_PIXELS; y++) {
      // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
      const auto paletteColor = pixelData[x * WIDTH_PIXELS + y];
      assert(paletteColor < palette->size());
      const auto sircColor = palette->at(paletteColor);
      // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
      output[x][y] = rgbaFromSircColor(sircColor);
    }
  }
  return output;
}
