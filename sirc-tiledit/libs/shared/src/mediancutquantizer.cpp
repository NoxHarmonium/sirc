#include "sircimage.hpp"

#include <mediancutquantizer.hpp>
#include <utils.hpp>

#include <algorithm>
#include <cassert>
#include <map>
#include <numeric>
#include <ranges>
#include <set>
#include <unordered_set>
#include <utility>

std::vector<SircColorComponent>
paletteAsSingleChannel(const std::span<const SircColor> &palette,
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
componentWiseAverageOfAllColors(const std::span<const SircColor> &palette) {
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

SircColorComponent findRangeOfChannel(const std::span<const SircColor> &palette,
                                      const ImageChannel channel) {

  std::vector<SircColorComponent> p = paletteAsSingleChannel(palette, channel);
  auto [min, max] = minmax_element(p.begin(), p.end());
  return *max - *min;
}

std::vector<SircColor>
paletteSortedByChannel(const std::span<const SircColor> &palette,
                       const ImageChannel channel) {
  std::vector output(palette.begin(), palette.end());
  std::ranges::stable_sort(
      output, [channel](const SircColor leftColor, const SircColor rightColor) {
        return std::less<SircColorComponent>{}(
            componentFromColor(leftColor, channel),
            componentFromColor(rightColor, channel));
      });
  return output;
}

ImageChannel
findChannelWithMostRange(const std::span<const SircColor> &originalPalette) {
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

std::vector<PaletteReference> buildPaletteMapping(
    const std::vector<std::pair<SircColor, SircColor>> &quantizedColorPairs,
    const std::span<const SircColor> &originalPalette,
    const std::span<const SircColor> &quantizedPalette) {
  std::vector<PaletteReference> out(originalPalette.size());

  const auto originalPaletteMap =
      spanToMapOfIndexes<PaletteReference>(originalPalette);
  const auto quantizedPaletteMap =
      spanToMapOfIndexes<PaletteReference>(quantizedPalette);

  for (const auto &[originalColor, quantizedColor] : quantizedColorPairs) {
    const PaletteReference originalIndex =
        findOrDefault(originalPaletteMap, originalColor);
    const PaletteReference newIndex =
        findOrDefault(quantizedPaletteMap, quantizedColor);
    out[originalIndex] = newIndex;
  }
  return out;
}

std::vector<SircColor> deduplicatePalette(
    std::vector<std::pair<SircColor, SircColor>> quantizedPalettePairs) {
  const auto quantizedPaletteValues = std::views::values(quantizedPalettePairs);
  const auto quantizedPaletteSet = std::unordered_set(
      quantizedPaletteValues.begin(), quantizedPaletteValues.end());
  return {quantizedPaletteSet.begin(), quantizedPaletteSet.end()};
}

// NOLINTNEXTLINE(misc-no-recursion)
void splitPaletteIntoBucketsAndAverage(
    const std::span<const SircColor> &palette,
    const std::span<std::pair<SircColor, SircColor>> &results,
    const size_t maxBucketSize) {
  if (palette.size() <= maxBucketSize) {
    const auto averageColor = componentWiseAverageOfAllColors(palette);
    std::ranges::transform(palette.begin(), palette.end(), results.begin(),
                           [averageColor](SircColor originalValue) {
                             return std::pair(originalValue, averageColor);
                           });
    return;
  }

  const auto channelWithMostRange = findChannelWithMostRange(palette);

  const auto sortedPalette =
      paletteSortedByChannel(palette, channelWithMostRange);
  const auto sortedPaletteSpan = std::span(sortedPalette);

  const long halfSize = static_cast<long>(sortedPalette.size() / 2);
  splitPaletteIntoBucketsAndAverage(sortedPaletteSpan.subspan(0, halfSize),
                                    results.subspan(0, halfSize),
                                    maxBucketSize);
  splitPaletteIntoBucketsAndAverage(sortedPaletteSpan.subspan(halfSize),
                                    results.subspan(halfSize), maxBucketSize);
}

SircImage MedianCutQuantizer::quantize(const SircImage &sircImage,
                                       const PaletteReductionBpp bpp) const {
  const auto maxPaletteSize = to_underlying(bpp);
  const auto [existingPalette, pixelData] = sircImage;

  if (existingPalette.size() <= maxPaletteSize) {
    return sircImage;
  }

  // Note: ceiling integer division
  const size_t maxBucketSize =
      (existingPalette.size() + maxPaletteSize - 1) / maxPaletteSize;

  auto results =
      std::vector<std::pair<SircColor, SircColor>>(existingPalette.size());
  splitPaletteIntoBucketsAndAverage(existingPalette, results, maxBucketSize);
  const auto quantizedPaletteWithoutDupes = deduplicatePalette(results);

  const auto paletteMapping =
      buildPaletteMapping(results, std::span(existingPalette),
                          std::span(quantizedPaletteWithoutDupes));

  SircImage quantizedImage = {.palette = quantizedPaletteWithoutDupes,
                              .pixelData = {}};

  std::ranges::transform(
      pixelData.cbegin(), pixelData.cend(), quantizedImage.pixelData.begin(),
      [paletteMapping](const PaletteReference &oldPaletteRef) {
        return paletteMapping[oldPaletteRef];
      });
  return quantizedImage;
}
