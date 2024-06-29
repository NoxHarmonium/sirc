#include "miscadapter.hpp"

#include <map>

SircImage MiscAdapter::packedSircPixelDataToSircImage(
    const PackedSircPixelData &pixelData) {

  SircImage imageData = {};
  std::map<SircColor, size_t> paletteLookup;

  for (int x = 0; x < WIDTH_PIXELS; x++) {
    for (int y = 0; y < HEIGHT_PIXELS; y++) {
      // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
      auto pixel = pixelData[x][y];

      if (auto existingPaletteIndex = paletteLookup.find(pixel);
          existingPaletteIndex != paletteLookup.end()) {
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        imageData.pixelData[x][y] = existingPaletteIndex->second;
      } else {
        imageData.palette.push_back(pixel);
        auto paletteIndex = imageData.palette.size() - 1;
        paletteLookup.insert({pixel, paletteIndex});
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        imageData.pixelData[x][y] = paletteIndex;
      }
    }
  }

  return imageData;
}
