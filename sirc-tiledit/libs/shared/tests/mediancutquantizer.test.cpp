#include <algorithm>
#include <vector>

#include "catch2/catch_amalgamated.hpp"

#include <iostream>
#include <mediancutquantizer.hpp>
#include <sircimage.hpp>

TEST_CASE("Single Image - Reduces palette size to 2bpp", "[quantize]") {
  const SircImage sircImage = {
      .palette = std::make_shared<std::vector<SircColor>>(
          std::initializer_list<SircColor>{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11}),
      .pixelData = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10}};

  const auto quantizer = MedianCutQuantizer();
  const auto quantizedImage =
      quantizer.quantize(sircImage, PaletteReductionBpp::TwoBpp);
  const auto [palette, pixelData] = quantizedImage;

  const std::vector<SircColor> expectedPalette = {0, 4, 7, 10};

  REQUIRE(4 == palette->size());
  REQUIRE((*palette)[0] == 0);
  REQUIRE(std::is_permutation(palette->cbegin(), palette->cend(),
                              expectedPalette.cbegin()));
  REQUIRE(
      std::ranges::all_of(pixelData, [&palette](const PaletteReference pixel) {
        return pixel < palette->size();
      }));
}

TEST_CASE("Single Image - Reduces palette size to 4bpp", "[quantize]") {
  const SircImage sircImage = {
      .palette = std::make_shared<std::vector<SircColor>>(
          std::initializer_list<SircColor>{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
                                           12, 13, 14, 15, 16, 17, 18}),
      .pixelData = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 17, 16, 15}};

  const auto quantizer = MedianCutQuantizer();
  const auto quantizedImage =
      quantizer.quantize(sircImage, PaletteReductionBpp::FourBpp);
  const auto [palette, pixelData] = quantizedImage;

  const std::vector<SircColor> expectedPalette = {
      0, 3, 5, 7, 8, 10, 12, 14, 16, 17,
  };

  REQUIRE(10 == palette->size());
  REQUIRE((*palette)[0] == 0);
  REQUIRE(std::is_permutation(palette->cbegin() + 1, palette->cend(),
                              expectedPalette.cbegin() + 1));
  REQUIRE(
      std::ranges::all_of(pixelData, [&palette](const PaletteReference pixel) {
        return pixel < palette->size();
      }));
}

TEST_CASE("Multiple Images - Reduces palette size to 2bpp", "[quantize]") {
  const SircImage sircImage1 = {
      .palette = std::make_shared<std::vector<SircColor>>(
          std::initializer_list<SircColor>{7, 8, 9, 10, 11, 7, 8, 9, 10, 11,
                                           12}),
      .pixelData = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10}};
  const SircImage sircImage2 = {
      .palette = std::make_shared<std::vector<SircColor>>(
          std::initializer_list<SircColor>{1, 2, 3, 4, 5, 6, 1, 2, 3, 4, 5, 6}),
      .pixelData = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10}};

  const auto quantizer = MedianCutQuantizer();
  const auto quantizedImages = quantizer.quantizeAll(
      {sircImage1, sircImage2}, PaletteReductionBpp::TwoBpp);

  for (const auto &quantizedImage : quantizedImages) {
    const auto [palette, pixelData] = quantizedImage;
    const std::vector<SircColor> expectedPalette = {0, 5, 8, 11};

    REQUIRE(4 == palette->size());
    REQUIRE((*palette)[0] == 0);
    REQUIRE(std::is_permutation(palette->cbegin(), palette->cend(),
                                expectedPalette.cbegin()));
    REQUIRE(std::ranges::all_of(pixelData,
                                [&palette](const PaletteReference pixel) {
                                  return pixel < palette->size();
                                }));
  }
}
