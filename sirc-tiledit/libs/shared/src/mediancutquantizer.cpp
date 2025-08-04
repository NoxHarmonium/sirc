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

/**
 * Takes a span of n palettes, each being associated with a different SircImage
 * struct, merges/de-duplicates all of them into a single palette and returns
 * a mapping that converts from each of the old palette indexes to index in the
 * new merged palette.
 *
 * @param palettes the span of palettes to merge
 * @return a pair where the first value is the new merged palette and the second
 * value is a vector of index mappings, where the indexes align with the
 * palettes provides in the palettes parameter.
 */
std::pair<std::vector<SircColor>, std::vector<std::vector<PaletteReference>>>
mergePalettesAndDeduplicate(
    const std::vector<std::vector<SircColor>> &palettes) {
  // TODO: Clean up this function
  // TODO: Should I use spans for the parameters?
  // TODO: Could I use spans for the return type (or would that cause issues
  // with ownership)
  // TODO: Is it wasteful to preallocate the vectors with empty values?
  //       Probably not since it probably just allocates a big chunk of empty
  //       memory
  auto results = std::vector<std::vector<PaletteReference>>(palettes.size());
  std::set<SircColor> mergedPalette;

  // Add all the palettes into a single set
  // All the palettes need to be inserted into the set in a single loop
  // before doing the remapping because the ordering would not be stable
  // between loop iterations
  for (auto const &palette : palettes) {
    // Insert the whole palette into the mergedPalette set to remove any
    // duplicates
    mergedPalette.insert(palette.begin(), palette.end());
  }

  for (auto [index, palette] : enumerate(palettes)) {
    // Allocate the inner vector
    results[index] = std::vector<PaletteReference>(palette.size());

    // Iterate through every colour in the palette to generate the mapping
    for (auto const [oldPaletteIndex, paletteEntry] : enumerate(palette)) {
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
quantize_palette_and_generate_mapping(
    const std::span<const SircColor> &existingPalette,
    const size_t maxPaletteSize) {

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

// TODO: use consistent casing for things (I think camelCase is the norm in
// this project)
SircImage transform_sirc_image_pixels_with_mapping(
    const SircImage &sircImage,
    const std::vector<SircColor> &quantizedPaletteWithoutDupes,
    const std::vector<PaletteReference> &paletteMapping) {
  const auto [existingPalette, pixelData] = sircImage;
  SircImage quantizedImage = {.palette = quantizedPaletteWithoutDupes,
                              .pixelData = {}};

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
  for (auto const [index, colour] : enumerate(paletteMapping)) {
    mergedPaletteMapping.push_back(paletteMapping2.at(colour));
  }

  return mergedPaletteMapping;
}

SircImage MedianCutQuantizer::quantize(const SircImage &sircImage,
                                       const PaletteReductionBpp bpp) const {
  const auto maxPaletteSize = to_underlying(bpp);
  const auto [existingPalette, pixelData] = sircImage;

  if (existingPalette.size() <= maxPaletteSize) {
    return sircImage;
  }

  const auto [quantizedPaletteWithoutDupes, paletteMapping] =
      quantize_palette_and_generate_mapping(existingPalette, maxPaletteSize);

  return transform_sirc_image_pixels_with_mapping(
      sircImage, quantizedPaletteWithoutDupes, paletteMapping);
}

std::vector<SircImage>
MedianCutQuantizer::quantize_all(const std::vector<SircImage> &sircImages,
                                 const PaletteReductionBpp bpp) const {
  const auto maxPaletteSize = to_underlying(bpp);

  std::vector<SircColor> existingPalette;

  // TODO: Extract this somewhere or use a view?
  std::vector<std::vector<SircColor>> palettes;
  palettes.reserve(sircImages.size());
  for (const auto &[palette, _] : sircImages) {
    palettes.push_back(palette);
  }

  // TODO: Can we make the palette const? Nothing passed in will be modified
  const auto [mergedPalette, mergedPaletteMappings] =
      mergePalettesAndDeduplicate(palettes);

  const auto [quantizedPalette, quantizedPaletteMapping] =
      quantize_palette_and_generate_mapping(mergedPalette, maxPaletteSize);

  std::vector<SircImage> output;
  output.reserve(sircImages.size());
  for (auto const [index, sircImage] : enumerate(sircImages)) {
    const auto mergedPaletteMapping = mergePaletteMappings(
        mergedPaletteMappings[index], quantizedPaletteMapping);

    SircImage quantizedImage = transform_sirc_image_pixels_with_mapping(
        sircImage, quantizedPalette, mergedPaletteMapping);
    output.push_back(quantizedImage);
  }

  return output;
}