#include "mediancutquantizer.h"

#include <algorithm>
#include <ranges>
#include <set>

enum class ImageChannel { R, G, B };

std::vector<SircColor>
paletteAsSingleChannel(const std::vector<SircColor> &palette,
                       const ImageChannel channel) {

  std::vector<SircColor> paletteAsSingleChannel;
  std::transform(palette.begin(), palette.end(),
                 std::back_inserter(paletteAsSingleChannel),
                 [channel](SircColor sircColor) {
                   switch (channel) {
                   case ImageChannel::R:
                     return (sircColor >> (SIRC_COLOR_COMPONENT_BITS * 2)) &
                            SIRC_COLOR_RANGE;
                   case ImageChannel::G:
                     return (sircColor >> SIRC_COLOR_COMPONENT_BITS) &
                            SIRC_COLOR_RANGE;
                   case ImageChannel::B:
                     return sircColor & SIRC_COLOR_RANGE;
                   }
                   throw std::runtime_error("Invalid ImageChannel value");
                 });
  return paletteAsSingleChannel;
}

SircColor averageColors(const std::vector<SircColor> &palette) {
  std::vector<SircColor> r = paletteAsSingleChannel(palette, ImageChannel::R);
  std::vector<SircColor> g = paletteAsSingleChannel(palette, ImageChannel::G);
  std::vector<SircColor> b = paletteAsSingleChannel(palette, ImageChannel::B);

  auto rAverage = std::accumulate(r.begin(), r.end(), 0) / r.size();
  auto gAverage = std::accumulate(g.begin(), g.end(), 0) / g.size();
  auto bAverage = std::accumulate(b.begin(), b.end(), 0) / b.size();

  return (rAverage << (SIRC_COLOR_COMPONENT_BITS * 2)) |
         (gAverage << SIRC_COLOR_COMPONENT_BITS) | bAverage;
}

unsigned short findRangeOfChannel(const std::vector<SircColor> &palette,
                                  const ImageChannel channel) {

  std::vector<SircColor> p = paletteAsSingleChannel(palette, channel);
  auto minmax = minmax_element(p.begin(), p.end());
  return minmax.second - minmax.first;
}

std::vector<SircColor>
sortPaletteByChannel(const std::vector<SircColor> &palette,
                     const ImageChannel channel) {
  auto output = palette;
  std::sort(
      output.begin(), output.end(),
      [channel](SircColor leftColor, SircColor rightColor) {
        switch (channel) {
        case ImageChannel::R: {
          auto a =
              (leftColor >> (SIRC_COLOR_COMPONENT_BITS * 2)) & SIRC_COLOR_RANGE;
          auto b = (rightColor >> (SIRC_COLOR_COMPONENT_BITS * 2)) &
                   SIRC_COLOR_RANGE;
          return a < b;
        }
        case ImageChannel::G: {
          auto a = (leftColor >> SIRC_COLOR_COMPONENT_BITS) & SIRC_COLOR_RANGE;
          auto b = (rightColor >> SIRC_COLOR_COMPONENT_BITS) & SIRC_COLOR_RANGE;
          return a < b;
        }
        case ImageChannel::B: {
          auto a = leftColor & SIRC_COLOR_RANGE;
          auto b = rightColor & SIRC_COLOR_RANGE;
          return a < b;
        }
        }
        throw std::runtime_error("Invalid ImageChannel value");
      });
  return output;
}

std::vector<std::pair<SircColor, SircColor>>
quantizeRecurse(const std::vector<SircColor> &originalPalette,
                const unsigned short maxBucketSize) {
  if (originalPalette.size() <= maxBucketSize) {
    auto average = averageColors(originalPalette);
    std::vector<std::pair<SircColor, SircColor>> paired;
    std::transform(originalPalette.begin(), originalPalette.end(),
                   std::back_inserter(paired),
                   [average](SircColor originalColor) {
                     return std::pair(originalColor, average);
                   });
    return paired;
  }

  const auto rRange = findRangeOfChannel(originalPalette, ImageChannel::R);
  const auto gRange = findRangeOfChannel(originalPalette, ImageChannel::G);
  const auto bRange = findRangeOfChannel(originalPalette, ImageChannel::B);

  const auto maxRange = std::max({rRange, gRange, bRange});

  ImageChannel channelWithMostRange = {};
  if (maxRange == rRange) {
    channelWithMostRange = ImageChannel::R;
  } else if (maxRange == rRange) {
    channelWithMostRange = ImageChannel::G;
  } else {
    channelWithMostRange = ImageChannel::B;
  }

  auto sortedPalette =
      sortPaletteByChannel(originalPalette, channelWithMostRange);

  const unsigned short halfSize = sortedPalette.size() / 2;
  std::vector<SircColor> lowerPalette(sortedPalette.begin(),
                                      sortedPalette.begin() + halfSize);
  std::vector<SircColor> upperPalette(sortedPalette.begin() + halfSize,
                                      sortedPalette.end());
  auto lowerQuantized = quantizeRecurse(lowerPalette, maxBucketSize);
  auto upperQuantized = quantizeRecurse(upperPalette, maxBucketSize);

  // Concatinate result
  std::vector<std::pair<SircColor, SircColor>> out(lowerQuantized.begin(),
                                                   lowerQuantized.end());
  out.insert(out.end(), upperQuantized.begin(), upperQuantized.end());
  return out;
}

std::map<PaletteReference, PaletteReference> buildPaletteMapping(
    std::vector<std::pair<SircColor, SircColor>> quantizedColorPairs,
    std::vector<SircColor> originalPalette,
    std::vector<SircColor> quantizedPalette) {
  std::map<PaletteReference, PaletteReference> out;
  for (auto &pair : quantizedColorPairs) {
    SircColor originalColor = pair.first;
    SircColor quantizedColor = pair.second;
    auto originalIndexIt = std::find(originalPalette.begin(),
                                     originalPalette.end(), originalColor);
    auto newIndexIt = std::find(quantizedPalette.begin(),
                                quantizedPalette.end(), quantizedColor);
    assert(originalIndexIt != originalPalette.end() &&
           newIndexIt != quantizedPalette.end());

    PaletteReference originalIndex = originalIndexIt - originalPalette.begin();
    PaletteReference newIndex = newIndexIt - quantizedPalette.begin();

    out[originalIndex] = newIndex;
  }
  return out;
}

SircImage MedianCutQuantizer::quantize(const SircImage &sircImage,
                                       const PaletteReductionBpp bpp) const {
  unsigned short maxPaletteSize = {};
  switch (bpp) {
  case PaletteReductionBpp::None:
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-magic-numbers)
    maxPaletteSize = 256;
    break;
  case PaletteReductionBpp::FourBpp:
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-magic-numbers)
    maxPaletteSize = 16;
    break;
  case PaletteReductionBpp::TwoBpp:
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-magic-numbers)
    maxPaletteSize = 4;
    break;
  }

  const auto originalPixelData = sircImage.getImageData().pixelData;
  const auto originalPalette = sircImage.getImageData().palette;

  qInfo("Quantizing image with palette size %zu to maxPaletteSize: %hu",
        originalPalette.size(), maxPaletteSize);

  if (originalPalette.size() <= maxPaletteSize) {
    return sircImage;
  }

  const unsigned short maxBucketSize =
      (originalPalette.size() + maxPaletteSize - 1) / maxPaletteSize;

  auto quantizedPalettePairs = quantizeRecurse(originalPalette, maxBucketSize);

  auto quantizedPaletteWithDupes = std::views::values(quantizedPalettePairs);
  auto quantizedPaletteSet = std::set(quantizedPaletteWithDupes.begin(),
                                      quantizedPaletteWithDupes.end());
  auto quantizedPaletteWithoutDupes = std::vector<SircColor>(
      quantizedPaletteSet.begin(), quantizedPaletteSet.end());

  auto paletteMapping = buildPaletteMapping(
      quantizedPalettePairs, originalPalette, quantizedPaletteWithoutDupes);

  SircImageData quantizedImage = {.palette = quantizedPaletteWithoutDupes,
                                  .pixelData = {}};

  for (int x = 0; x < WIDTH_PIXELS; x++) {
    for (int y = 0; y < HEIGHT_PIXELS; y++) {
      // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
      quantizedImage.pixelData[x][y] = paletteMapping[originalPixelData[x][y]];
    }
  }

  return SircImage::fromSircImageData(quantizedImage);
}
