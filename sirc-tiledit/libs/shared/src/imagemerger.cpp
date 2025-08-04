#include <imagemerger.hpp>

#include <algorithm>
#include <format>

SircImage ImageMerger::merge(const std::vector<SircImage> &inputImages) {
  SircImage result = {};

  if (inputImages.empty()) {
    return result;
  }

  const auto firstImageSize = inputImages.front().pixelData.size();
  const auto firstPalette = inputImages.front().palette;
  result.palette = firstPalette;

  for (const auto &[palette, pixelData] : inputImages) {
    if (palette != firstPalette) {
      throw std::invalid_argument("All palettes must match");
    }
    if (pixelData.size() != firstImageSize) {
      // Can't be currently thrown because pixelData is an array of fixed size
      // initialised to zeros. However, if this changes in the future this will
      // pick it up
      throw std::invalid_argument("All input images must be the same size");
    }

    // TODO: Do some benchmarking around this function to see what the optimum
    // order of operations is
    //       (E.g. maybe iterating over each input images at once could be
    //       faster than a pass for each input image)
    std::ranges::transform(
        result.pixelData, pixelData, result.pixelData.begin(),
        [firstPalette](const PaletteReference &current,
                       const PaletteReference &candidate) {
          // Only update if candidate is non-transparent
          if (candidate >= firstPalette.size()) {
            throw std::invalid_argument(std::format(
                "Pixel value {} is out of bounds of the palette of size {}",
                candidate, firstPalette.size()));
          }
          if (const auto resolvedCandidate = firstPalette[candidate];
              resolvedCandidate != 0) {
            return candidate;
          }
          return current; // Keep existing value
        });
  }

  return result;
}