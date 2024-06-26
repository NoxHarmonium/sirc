#include "sircimage.hpp"
#include <cstdio>
#include <mediancutquantizer.hpp>

#include <algorithm>
#include <cassert>
#include <map>
#include <miscadapter.hpp>
#include <numeric>
#include <ranges>
#include <set>

enum class ImageChannel : std::uint8_t { R, G, B };

std::vector<SircColorComponent>
paletteAsSingleChannel(const std::vector<SircColor> &palette,
                       const ImageChannel channel) {

  std::vector<SircColorComponent> paletteAsSingleChannel;
  std::ranges::transform(
      palette, std::back_inserter(paletteAsSingleChannel),
      [channel](const SircColor sircColor) {
        switch (channel) {
        case ImageChannel::R:
          return sircColor >> SIRC_COLOR_COMPONENT_BITS * 2 & SIRC_COLOR_RANGE;
        case ImageChannel::G:
          return sircColor >> SIRC_COLOR_COMPONENT_BITS & SIRC_COLOR_RANGE;
        case ImageChannel::B:
          return sircColor & SIRC_COLOR_RANGE;
        }
        throw std::runtime_error("Invalid ImageChannel value");
      });
  return paletteAsSingleChannel;
}

SircColor averageColors(const std::vector<SircColor> &palette) {
  const std::vector<SircColorComponent> r =
      paletteAsSingleChannel(palette, ImageChannel::R);
  const std::vector<SircColorComponent> g =
      paletteAsSingleChannel(palette, ImageChannel::G);
  const std::vector<SircColorComponent> b =
      paletteAsSingleChannel(palette, ImageChannel::B);

  auto const count = static_cast<float>(r.size());
  auto const averageOfComponent =
      [&count](std::vector<SircColorComponent> component) {
        auto const result =
            std::reduce(component.begin(), component.end(), 0ul) /
            static_cast<unsigned long>(count);
        assert(result <= SIRC_COLOR_RANGE);
        return static_cast<SircColorComponent>(result);
      };

  const SircColorComponent rAverage = averageOfComponent(r);
  const SircColorComponent gAverage = averageOfComponent(g);
  const SircColorComponent bAverage = averageOfComponent(b);

  return rAverage << SIRC_COLOR_COMPONENT_BITS * 2 |
         gAverage << SIRC_COLOR_COMPONENT_BITS | bAverage;
}

SircColorComponent findRangeOfChannel(const std::vector<SircColor> &palette,
                                      const ImageChannel channel) {

  std::vector<SircColorComponent> p = paletteAsSingleChannel(palette, channel);
  auto [min, max] = minmax_element(p.begin(), p.end());
  return *max - *min;
}

std::vector<SircColor>
sortPaletteByChannel(const std::vector<SircColor> &palette,
                     const ImageChannel channel) {
  auto output = palette;
  std::ranges::stable_sort(output, [channel](const SircColor leftColor,
                                             const SircColor rightColor) {
    switch (channel) {
    case ImageChannel::R: {
      const auto a =
          leftColor >> SIRC_COLOR_COMPONENT_BITS * 2 & SIRC_COLOR_RANGE;
      const auto b =
          rightColor >> SIRC_COLOR_COMPONENT_BITS * 2 & SIRC_COLOR_RANGE;
      assert(a <= SIRC_COLOR_RANGE && b <= SIRC_COLOR_RANGE);
      return a < b;
    }
    case ImageChannel::G: {
      const auto a = leftColor >> SIRC_COLOR_COMPONENT_BITS & SIRC_COLOR_RANGE;
      const auto b = rightColor >> SIRC_COLOR_COMPONENT_BITS & SIRC_COLOR_RANGE;
      assert(a <= SIRC_COLOR_RANGE && b <= SIRC_COLOR_RANGE);
      return a < b;
    }
    case ImageChannel::B: {
      const auto a = leftColor & SIRC_COLOR_RANGE;
      const auto b = rightColor & SIRC_COLOR_RANGE;
      assert(a <= SIRC_COLOR_RANGE && b <= SIRC_COLOR_RANGE);
      return a < b;
    }
    }
    throw std::runtime_error("Invalid ImageChannel value");
  });
  return output;
}

ImageChannel
findChannelWithMostRange(const std::vector<SircColor> &originalPalette) {
  const auto rRange = findRangeOfChannel(originalPalette, ImageChannel::R);
  const auto gRange = findRangeOfChannel(originalPalette, ImageChannel::G);
  const auto bRange = findRangeOfChannel(originalPalette, ImageChannel::B);

  const auto maxRange = std::max({rRange, gRange, bRange});
  assert(maxRange <= SIRC_COLOR_RANGE);

  if (maxRange == rRange) {
    return ImageChannel::R;
  }
  if (maxRange == gRange) {
    return ImageChannel::G;
  }
  return ImageChannel::B;
}
// TODO: Consider removing recursion in MedianCutQuantizer
// category=Refactoring
// It is probably better to make this non recursive (or tail recursive) but it
// works for now so maybe ok
std::vector<std::pair<SircColor, SircColor>>
// NOLINTNEXTLINE(misc-no-recursion)
quantizeRecurse(const std::vector<SircColor> &originalPalette,
                const size_t maxBucketSize) {
  if (originalPalette.size() <= maxBucketSize) {
    auto average = averageColors(originalPalette);
    std::vector<std::pair<SircColor, SircColor>> paired;
    std::ranges::transform(originalPalette, std::back_inserter(paired),
                           [average](SircColor originalColor) {
                             return std::pair(originalColor, average);
                           });
    return paired;
  }

  const auto channelWithMostRange = findChannelWithMostRange(originalPalette);

  auto sortedPalette =
      sortPaletteByChannel(originalPalette, channelWithMostRange);

  const long halfSize = static_cast<long>(sortedPalette.size() / 2);
  const std::vector lowerPalette(sortedPalette.begin(),
                                 sortedPalette.begin() + halfSize);
  const std::vector upperPalette(sortedPalette.begin() + halfSize,
                                 sortedPalette.end());
  auto lowerQuantized = quantizeRecurse(lowerPalette, maxBucketSize);
  auto upperQuantized = quantizeRecurse(upperPalette, maxBucketSize);

  // Concatinate result
  std::vector out(lowerQuantized.begin(), lowerQuantized.end());
  out.insert(out.end(), upperQuantized.begin(), upperQuantized.end());
  return out;
}

std::map<PaletteReference, PaletteReference> buildPaletteMapping(
    const std::vector<std::pair<SircColor, SircColor>> &quantizedColorPairs,
    std::vector<SircColor> originalPalette,
    std::vector<SircColor> quantizedPalette) {
  std::map<PaletteReference, PaletteReference> out;
  for (const auto &[originalColor, quantizedColor] : quantizedColorPairs) {
    const auto originalIndexIt =
        std::ranges::find(originalPalette, originalColor);
    const auto newIndexIt = std::ranges::find(quantizedPalette, quantizedColor);
    assert(originalIndexIt != originalPalette.end() &&
           newIndexIt != quantizedPalette.end());

    const PaletteReference originalIndex =
        originalIndexIt - originalPalette.begin();
    const PaletteReference newIndex = newIndexIt - quantizedPalette.begin();

    out[originalIndex] = newIndex;
  }
  return out;
}

SircImage MedianCutQuantizer::quantize(const SircImage &sircImage,
                                       const PaletteReductionBpp bpp) const {
  size_t maxPaletteSize = {};
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

  const auto originalPixelData = sircImage.pixelData;
  const auto originalPalette = sircImage.palette;

  if (originalPalette.size() <= maxPaletteSize) {
    return sircImage;
  }

  const size_t maxBucketSize =
      (originalPalette.size() + maxPaletteSize - 1) / maxPaletteSize;

  auto quantizedPalettePairs = quantizeRecurse(originalPalette, maxBucketSize);

  auto quantizedPaletteWithDupes = std::views::values(quantizedPalettePairs);
  auto quantizedPaletteSet = std::set(quantizedPaletteWithDupes.begin(),
                                      quantizedPaletteWithDupes.end());
  const auto quantizedPaletteWithoutDupes =
      std::vector(quantizedPaletteSet.begin(), quantizedPaletteSet.end());

  auto paletteMapping = buildPaletteMapping(
      quantizedPalettePairs, originalPalette, quantizedPaletteWithoutDupes);

  SircImage quantizedImage = {.palette = quantizedPaletteWithoutDupes,
                              .pixelData = {}};

  for (int x = 0; x < WIDTH_PIXELS; x++) {
    for (int y = 0; y < HEIGHT_PIXELS; y++) {
      // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
      quantizedImage.pixelData[x][y] = paletteMapping[originalPixelData[x][y]];
    }
  }

  return quantizedImage;
}
