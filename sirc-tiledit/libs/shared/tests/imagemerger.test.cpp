#include "imagemerger.hpp"

#include <algorithm>
#include <vector>

#include "catch2/catch_amalgamated.hpp"

#include <ranges>
#include <sircimage.hpp>

TEST_CASE("Merges images correctly", "[preview]") {
  const SircImage sircImage1 = {
      // First palette (no transparent palette entry)          -
      .palette = {100, 101, 102},
      // Pixel data: [100, 101, 100, 102, 100]
      .pixelData = {0, 1, 0, 2, 0}};
  const SircImage sircImage2 = {
      // Second palette (with transparency)
      .palette = {0, 201, 202},
      // Pixel data: [201, transparent, 202, transparent, transparent]
      .pixelData = {1, 0, 2, 0, 0}};
  const SircImage sircImage3 = {
      // Third palette (with transparency)
      .palette = {0, 301, 302},
      // Pixel data: [transparent, transparent, transparent, transparent, 301]
      .pixelData = {0, 0, 0, 0, 1}};

  auto [palette, pixelData] =
      ImageMerger::merge({sircImage1, sircImage2, sircImage3});

  // Expected combined palette (concatenation of all palettes)
  std::vector<SircColor> expectedPalette = {
      100, 101, 102, // From image1
      0,   201, 202, // From image2
      0,   301, 302  // From image3
  };

  // 201, 101, 201, 102, 301
  IndexedPixelData expectedPixels = {4, 1, 5, 2, 7};

  REQUIRE(pixelData == expectedPixels);
  REQUIRE(palette == expectedPalette);
}