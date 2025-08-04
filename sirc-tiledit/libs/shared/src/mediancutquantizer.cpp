#include "sircimage.hpp"

#include <mediancutquantizer.hpp>
#include <utils.hpp>

#include <__ostream/basic_ostream.h>
#include <algorithm>
#include <cassert>
#include <iostream>
#include <map>
#include <numeric>
#include <ranges>
#include <set>
#include <unordered_set>
#include <utility>
#include <valarray>

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

std::vector<std::valarray<SircColorComponent>>
paletteAsSingleChannels(const std::span<const SircColor> &palette) {
  std::vector<std::valarray<SircColorComponent>> paletteAsSingleChannelsOut;
  paletteAsSingleChannelsOut.reserve(palette.size());
  std::ranges::transform(
      palette, std::back_inserter(paletteAsSingleChannelsOut),
      [](const SircColor sircColor) -> std::valarray<SircColorComponent> {
        return {componentFromColor(sircColor, ImageChannel::R),
                componentFromColor(sircColor, ImageChannel::G),
                componentFromColor(sircColor, ImageChannel::B)};
      });
  return paletteAsSingleChannelsOut;
}

SircColor
componentWiseAverageOfAllColors(const std::span<const SircColor> &palette) {
  auto const channels = paletteAsSingleChannels(palette);

  const std::valarray initial = {0ul, 0ul, 0ul};
  auto const sum = std::reduce(channels.cbegin(), channels.cend(), initial,
                               [](const std::valarray<unsigned long> &acc,
                                  const std::valarray<SircColorComponent> &v)
                                   -> std::valarray<unsigned long> {
                                 auto const casted = std::valarray{
                                     static_cast<unsigned long>(v[0]),
                                     static_cast<unsigned long>(v[1]),
                                     static_cast<unsigned long>(v[2])};
                                 return acc + casted;
                               });

  auto const average = sum / channels.size();

  assert(average[0] <= SIRC_COLOR_RANGE && average[1] <= SIRC_COLOR_RANGE &&
         average[2] <= SIRC_COLOR_RANGE);

  return colorFromComponent(average[0], ImageChannel::R) |
         colorFromComponent(average[1], ImageChannel::G) |
         colorFromComponent(average[2], ImageChannel::B);
}

SircColorComponent findRangeOfChannel(const std::span<const SircColor> &palette,
                                      const ImageChannel channel) {
  // Future work: maybe we could have a `findRangeOfChannels` that does all
  // three channels in one iteration, theoretically that would be faster
  std::vector<SircColorComponent> p = paletteAsSingleChannel(palette, channel);
  auto [min, max] = std::ranges::minmax_element(p.begin(), p.end());
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
  const auto quantizedPaletteValues =
      quantizedPalettePairs | std::views::values;
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

/**
 * Takes a span of n SircImage structs and merges/de-duplicates all of their
 * palettes into a single palette and returns a mapping that converts from each
 * of the old palette indexes to index in the new merged palette.
 *
 * @param sircImages the span of SircImage structs with palettes to merge
 * @return a pair where the first element is the merged/deduplicated palette and
 * the second element is a mapping between the old palette index and the new
 * palette indexes
 */
std::pair<std::vector<SircColor>, std::vector<std::vector<PaletteReference>>>
mergePalettesAndDeduplicate(const std::vector<SircImage> &sircImages) {
  auto results = std::vector<std::vector<PaletteReference>>(sircImages.size());
  std::set<SircColor> mergedPalette;

  // Add all the palettes into a single set
  // All the palettes need to be inserted into the set in a single loop
  // before doing the remapping because the ordering would not be stable
  // between loop iterations
  for (auto const &[palette, _] : sircImages) {
    // Insert the whole palette into the mergedPalette set to remove any
    // duplicates
    mergedPalette.insert(palette->begin(), palette->end());
  }

  for (auto const &[index, sircImage] : enumerate(sircImages)) {
    auto const &[palette, _] = sircImage;
    // Allocate the inner vector
    results[index] = std::vector<PaletteReference>(palette->size());

    // Iterate through every colour in the palette to generate the mapping
    for (auto const [oldPaletteIndex, paletteEntry] : enumerate(*palette)) {
      // Find where the colour is situated in the set (to map the old index to
      // the new index)
      auto it3 = mergedPalette.find(paletteEntry);
      // The colour will always be in the set, unless there is a coding error,
      // so an assertion is probably good enough here
      assert(it3 != mergedPalette.end());
      auto const newIndex = std::distance(mergedPalette.begin(), it3);
      // Map the old palette index to the new palette index
      results[index][oldPaletteIndex] = newIndex;
    }
  }
  return {std::vector(mergedPalette.cbegin(), mergedPalette.cend()), results};
}

std::tuple<std::vector<SircColor>, std::vector<PaletteReference>>
quantizePaletteAndGenerateMapping(
    const std::span<const SircColor> &existingPalette,
    const size_t maxPaletteSize) {
  assert(maxPaletteSize > 0);

  // Note: ceiling integer division
  const size_t maxBucketSize =
      (existingPalette.size() + maxPaletteSize - 1) / maxPaletteSize;

  auto results =
      std::vector<std::pair<SircColor, SircColor>>(existingPalette.size());
  splitPaletteIntoBucketsAndAverage(existingPalette, results, maxBucketSize);
  const auto quantizedPaletteWithoutDupes = deduplicatePalette(results);

  return {quantizedPaletteWithoutDupes,
          buildPaletteMapping(results, std::span(existingPalette),
                              std::span(quantizedPaletteWithoutDupes))};
}

SircImage transformSircImagePixelsWithMapping(
    const SircImage &sircImage,
    const std::vector<SircColor> &quantizedPaletteWithoutDupes,
    const std::vector<PaletteReference> &paletteMapping) {
  const auto [existingPalette, pixelData] = sircImage;
  SircImage quantizedImage = {
      .palette = std::make_shared<std::vector<SircColor>>(
          quantizedPaletteWithoutDupes),
      .pixelData = {},

  };

  std::ranges::transform(
      pixelData.cbegin(), pixelData.cend(), quantizedImage.pixelData.begin(),
      [paletteMapping](const PaletteReference &oldPaletteRef) {
        return paletteMapping[oldPaletteRef];
      });
  return quantizedImage;
}

std::vector<PaletteReference>
mergePaletteMappings(const std::vector<PaletteReference> &paletteMapping,
                     const std::vector<PaletteReference> &paletteMapping2) {
  std::vector<PaletteReference> mergedPaletteMapping;
  mergedPaletteMapping.reserve(paletteMapping.size());
  std::ranges::transform(paletteMapping,
                         std::back_inserter(mergedPaletteMapping),
                         [&paletteMapping2](const PaletteReference &color) {
                           return paletteMapping2.at(color);
                         });

  return mergedPaletteMapping;
}

SircImage MedianCutQuantizer::quantize(const SircImage &sircImage,
                                       const PaletteReductionBpp bpp) const {
  const auto maxPaletteSize = to_underlying(bpp);
  const auto [existingPalette, pixelData] = sircImage;

  if (existingPalette->size() <= maxPaletteSize) {
    return sircImage;
  }

  const auto [quantizedPaletteWithoutDupes, paletteMapping] =
      quantizePaletteAndGenerateMapping(*existingPalette, maxPaletteSize);

  return transformSircImagePixelsWithMapping(
      sircImage, quantizedPaletteWithoutDupes, paletteMapping);
}

std::vector<SircImage>
MedianCutQuantizer::quantizeAll(const std::vector<SircImage> &sircImages,
                                const PaletteReductionBpp bpp) const {
  const auto maxPaletteSize = to_underlying(bpp);

  const auto [mergedPalette, mergedPaletteMappings] =
      mergePalettesAndDeduplicate(sircImages);

  const auto [quantizedPalette, quantizedPaletteMapping] =
      quantizePaletteAndGenerateMapping(mergedPalette, maxPaletteSize);

  std::vector<SircImage> output;
  output.reserve(sircImages.size());
  for (auto const [index, sircImage] : enumerate(sircImages)) {
    const auto mergedPaletteMapping = mergePaletteMappings(
        mergedPaletteMappings[index], quantizedPaletteMapping);

    SircImage quantizedImage = transformSircImagePixelsWithMapping(
        sircImage, quantizedPalette, mergedPaletteMapping);
    output.push_back(quantizedImage);
  }

  return output;
}