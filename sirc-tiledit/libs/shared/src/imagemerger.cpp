#include <imagemerger.hpp>

#include <algorithm>

SircImage ImageMerger::merge(const std::vector<SircImage> &inputImages) {
  // This is useful for previewing multiple layers in the UI, but it just merged
  // together the palettes in a dumb way that won't work when exporting because
  // a tileset needs to be paired with a single 16 colour palette.
  SircImage result = {};
  if (inputImages.empty()) {
    return result;
  }
  const auto firstImageSize = inputImages.front().pixelData.size();

  for (const auto &[palette, pixelData] : inputImages) {
    if (pixelData.size() != firstImageSize) {
      throw std::invalid_argument("All input images must be the same size");
    }

    auto const resultPalette = result.palette;
    auto const sourceImagePalette = palette;
    // TODO: Do some benchmarking around this function to see what the optimum
    // order of operations is
    //       (E.g. maybe iterating over each input images at once could be
    //       faster than a pass for each input image)
    std::transform(
        result.pixelData.cbegin(), result.pixelData.cend(), pixelData.cbegin(),
        result.pixelData.begin(),
        [resultPalette, sourceImagePalette](const PaletteReference &current,
                                            const PaletteReference &candidate) {
          // Only update if candidate is non-transparent
          if (const auto resolvedCandidate = sourceImagePalette[candidate];
              resolvedCandidate != 0) {
            const auto paletteOffset = resultPalette.size();
            return candidate + paletteOffset;
          }
          return current; // Keep existing value
        });

    // Insert the palette after
    result.palette.insert(result.palette.cend(), palette.cbegin(),
                          palette.cend());
  }

  return result;
}