#include "sircimage.hpp"

#include <mediancutquantizer.hpp>
#include <utils.hpp>

#include <algorithm>
#include <cassert>
#include <numeric>
#include <ranges>
#include <set>
#include <unordered_map>

std::vector<SircColorComponent>
paletteAsSingleChannel(const std::vector<SircColor> &palette,
                       const ImageChannel channel) {

  std::vector<SircColorComponent> paletteAsSingleChannel;
  paletteAsSingleChannel.reserve(palette.size());
  std::ranges::transform(palette, std::back_inserter(paletteAsSingleChannel),
                         [channel](const SircColor sircColor) {
                           return componentFromColor(sircColor, channel);
                         });
  return paletteAsSingleChannel;
}

SircColor
componentWiseAverageOfAllColors(const std::vector<SircColor> &palette) {
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

  return colorFromComponent(averageOfComponent(r), ImageChannel::R) |
         colorFromComponent(averageOfComponent(g), ImageChannel::G) |
         colorFromComponent(averageOfComponent(b), ImageChannel::B);
}

SircColorComponent findRangeOfChannel(const std::vector<SircColor> &palette,
                                      const ImageChannel channel) {

  std::vector<SircColorComponent> p = paletteAsSingleChannel(palette, channel);
  auto [min, max] = minmax_element(p.begin(), p.end());
  return *max - *min;
}

std::vector<SircColor>
paletteSortedByChannel(const std::vector<SircColor> &palette,
                       const ImageChannel channel) {
  std::vector output(palette);
  std::ranges::stable_sort(
      output, [channel](const SircColor leftColor, const SircColor rightColor) {
        return std::less<SircColorComponent>{}(
            componentFromColor(leftColor, channel),
            componentFromColor(rightColor, channel));
      });
  return output;
}

ImageChannel
findChannelWithMostRange(const std::vector<SircColor> &originalPalette) {
  const auto rRange = findRangeOfChannel(originalPalette, ImageChannel::R);
  const auto gRange = findRangeOfChannel(originalPalette, ImageChannel::G);
  const auto bRange = findRangeOfChannel(originalPalette, ImageChannel::B);

  const auto maxRange = std::max({rRange, gRange, bRange});

  if (maxRange == rRange) {
    return ImageChannel::R;
  }
  if (maxRange == gRange) {
    return ImageChannel::G;
  }
  return ImageChannel::B;
}

std::unordered_map<PaletteReference, PaletteReference> buildPaletteMapping(
    const std::vector<std::pair<SircColor, SircColor>> &quantizedColorPairs,
    std::vector<SircColor> originalPalette,
    std::vector<SircColor> quantizedPalette) {
  std::unordered_map<PaletteReference, PaletteReference> out;
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

std::vector<SircColor> deduplicatePalette(
    std::vector<std::pair<SircColor, SircColor>> quantizedPalettePairs) {
  auto quantizedPaletteWithDupes = std::views::values(quantizedPalettePairs);
  auto quantizedPaletteSet = std::set(quantizedPaletteWithDupes.begin(),
                                      quantizedPaletteWithDupes.end());
  return {quantizedPaletteSet.begin(), quantizedPaletteSet.end()};
}

std::vector<std::pair<SircColor, SircColor>>
// NOLINTNEXTLINE(misc-no-recursion)
splitPaletteIntoBucketsAndAverage(const std::vector<SircColor> &palette,
                                  const size_t maxBucketSize) {
  if (palette.size() <= maxBucketSize) {
    const auto average = componentWiseAverageOfAllColors(palette);
    return pairWithValue(palette, average);
  }

  const auto channelWithMostRange = findChannelWithMostRange(palette);

  const auto sortedPalette =
      paletteSortedByChannel(palette, channelWithMostRange);

  const long halfSize = static_cast<long>(sortedPalette.size() / 2);
  const std::vector lowerPalette(sortedPalette.begin(),
                                 sortedPalette.begin() + halfSize);
  const std::vector upperPalette(sortedPalette.begin() + halfSize,
                                 sortedPalette.end());

  return concatVecs(
      splitPaletteIntoBucketsAndAverage(lowerPalette, maxBucketSize),
      splitPaletteIntoBucketsAndAverage(upperPalette, maxBucketSize));
}

SircImage MedianCutQuantizer::quantize(const SircImage &sircImage,
                                       const PaletteReductionBpp bpp) const {
  const auto maxPaletteSize = to_underlying(bpp);
  const auto [existingPalette, pixelData] = sircImage;

  if (existingPalette.size() <= maxPaletteSize) {
    return sircImage;
  }

  const size_t maxBucketSize =
      (existingPalette.size() + maxPaletteSize - 1) / maxPaletteSize;

  const auto quantizedPalettePairs =
      splitPaletteIntoBucketsAndAverage(existingPalette, maxBucketSize);
  const auto quantizedPaletteWithoutDupes =
      deduplicatePalette(quantizedPalettePairs);
  const auto paletteMapping = buildPaletteMapping(
      quantizedPalettePairs, existingPalette, quantizedPaletteWithoutDupes);

  SircImage quantizedImage = {.palette = quantizedPaletteWithoutDupes,
                              .pixelData = {}};

  std::ranges::transform(
      pixelData.cbegin(), pixelData.cend(), quantizedImage.pixelData.begin(),
      [paletteMapping](const PaletteReference &oldPaletteRef) {
        return findOrDefault(paletteMapping, oldPaletteRef);
      });
  return quantizedImage;
}
