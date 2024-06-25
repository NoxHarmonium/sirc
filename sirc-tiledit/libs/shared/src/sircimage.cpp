#include <cassert>
#include <sircimage.hpp>

SircImage::SircImage() = default;

SircImage SircImage::fromPixelData(const PackedPixelData &pixelData) {

  auto sircImage = SircImage();

  for (int x = 0; x < WIDTH_PIXELS; x++) {
    for (int y = 0; y < HEIGHT_PIXELS; y++) {
      // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
      auto pixel = pixelData[x][y];

      if (auto existingPaletteIndex = sircImage.paletteLookup.find(pixel);
          existingPaletteIndex != sircImage.paletteLookup.end()) {
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        sircImage.imageData.pixelData[x][y] = existingPaletteIndex->second;
      } else {
        sircImage.imageData.palette.push_back(pixel);
        auto paletteIndex = sircImage.imageData.palette.size() - 1;
        sircImage.paletteLookup.insert({pixel, paletteIndex});
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        sircImage.imageData.pixelData[x][y] = paletteIndex;
      }
    }
  }

  return sircImage;
}

SircImage SircImage::fromSircImageData(const SircImageData &imageData) {
  auto sircImage = SircImage();
  // TODO: Check if this is a copy
  sircImage.imageData = imageData;

  int i = 0;
  for (auto paletteColor : sircImage.imageData.palette) {
    sircImage.paletteLookup.insert({paletteColor, i++});
  }

  // qInfo("Total palette entries: %zu", sircImage.imageData.palette.size());

  return sircImage;
}

// TODO: This breaks encapsulation I suppose, possibly making this class kind of
// pointless. Might need to revisit
SircImageData SircImage::getImageData() const { return this->imageData; }
