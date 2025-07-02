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

  for (auto sourceImage : inputImages) {
    if (sourceImage.pixelData.size() != firstImageSize) {
      throw std::invalid_argument("All input images must be the same size");
    }
    auto paletteOffset = result.palette.size();
    result.palette.insert(result.palette.cend(), sourceImage.palette.cbegin(),
                          sourceImage.palette.cend());

    std::transform(result.pixelData.cbegin(), result.pixelData.cend(),
                   sourceImage.pixelData.cbegin(), result.pixelData.begin(),
                   [result, sourceImage, paletteOffset](
                       const SircColor &current, const SircColor &candidate) {
                     const auto resolvedCurrent = result.palette[current];
                     const auto resolvedCandidate =
                         sourceImage.palette[candidate];
                     // Only update if current pixel is transparent (0)
                     // and candidate is non-transparent
                     if (resolvedCurrent == 0 && resolvedCandidate != 0) {
                       return static_cast<SircColor>(candidate + paletteOffset);
                     }
                     return current; // Keep existing value
                   });
  }

  return result;
}