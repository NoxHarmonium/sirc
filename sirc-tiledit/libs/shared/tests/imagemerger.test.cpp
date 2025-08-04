#include "imagemerger.hpp"

#include <algorithm>
#include <vector>

#include "catch2/catch_amalgamated.hpp"

#include <ranges>
#include <sircimage.hpp>

TEST_CASE("Merges images correctly", "[preview]") {
  const SircImage sircImage1 = {
      .palette = {100, 101, 102, 0, 201, 202, 0, 301, 302},
      // Pixel data: [100, 101, 100, 102, 100]
      .pixelData = {0, 1, 0, 2, 0}};
  const SircImage sircImage2 = {
      .palette = {100, 101, 102, 0, 201, 202, 0, 301, 302},
      // Pixel data: [201, transparent, 202, transparent, transparent]
      .pixelData = {4, 3, 5, 3, 3}};
  const SircImage sircImage3 = {
      .palette = {100, 101, 102, 0, 201, 202, 0, 301, 302},
      // Pixel data: [transparent, transparent, transparent, transparent, 301]
      .pixelData = {6, 6, 6, 6, 7}};

  auto [palette, pixelData] =
      ImageMerger::merge({sircImage1, sircImage2, sircImage3});

  // Expected combined palette (all palettes should be equal)
  std::vector<SircColor> expectedPalette = {100, 101, 102, 0,  201,
                                            202, 0,   301, 302};

  // 201, 101, 201, 102, 301
  IndexedPixelData expectedPixels = {4, 1, 5, 2, 7};

  REQUIRE(pixelData == expectedPixels);
  REQUIRE(palette == expectedPalette);
}

TEST_CASE("Throws when pixel data is out of range for palette", "[preview]") {
  const SircImage sircImage1 = {.palette = {100, 101, 102},
                                // Pixel data: [100, 101, 100, 102, 100]
                                .pixelData = {0, 1, 0, 2, 0}};
  const SircImage sircImage2 = {
      .palette = {100, 101, 102},
      // Pixel data: [201, transparent, 202, transparent, transparent]
      .pixelData = {4, 3, 5}};

  REQUIRE_THROWS_AS(ImageMerger::merge({sircImage1, sircImage2}),
                    std::invalid_argument);
}

TEST_CASE("Does not merge images with different palettes", "[preview]") {
  const SircImage sircImage1 = {.palette = {100, 101, 102},
                                // Pixel data: [100, 101, 100, 102, 100]
                                .pixelData = {0, 1, 0, 2, 0}};
  const SircImage sircImage2 = {
      .palette = {0, 201, 202, 0, 301, 302},
      // Pixel data: [201, transparent, 202, transparent, transparent]
      .pixelData = {4, 3, 5, 3, 3}};

  REQUIRE_THROWS_AS(ImageMerger::merge({sircImage1, sircImage2}),
                    std::invalid_argument);
}