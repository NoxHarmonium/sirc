#include "imagemerger.hpp"

#include <algorithm>
#include <vector>

#include "catch2/catch_amalgamated.hpp"

#include <sircimage.hpp>

TEST_CASE("Merges images correctly", "[preview]") {
  // Test setup - each image has a unique palette and meaningful pixel data
  const SircImage sircImage1 = {
      .palette = {100, 101, 102}, // First palette
      .pixelData = {0, 1, 0, 2, 0}
      // Pixel data: [transparent, color1, transparent, color2, transparent]
  };
  const SircImage sircImage2 = {
      .palette = {200, 201, 202}, // Second palette
      .pixelData = {1, 0, 2, 0, 0}
      // Pixel data: [color1, transparent, color2, transparent, transparent]
  };

  const SircImage sircImage3 = {
      .palette = {300, 301, 302},  // Third palette
      .pixelData = {0, 0, 0, 0, 1} // Pixel data: [transparent, transparent,
                                   // transparent, transparent, color1]
  };

  auto result = ImageMerger::merge({sircImage1, sircImage2, sircImage3});

  // Expected combined palette (concatenation of all palettes)
  std::vector<SircColor> expectedPalette = {
      100, 101, 102, // From image1
      200, 201, 202, // From image2
      300, 301, 302  // From image3
  };

  // Expected pixel data:
  // Position 0: image1=0 (transparent) -> use image2's color1 (200) at offset 3
  // -> 1+3=4 Position 1: image1's color1 (101) -> kept at original offset 1
  // Position 2: image1=0 -> use image2's color2 (202) at offset 3 -> 2+3=5
  // Position 3: image1's color2 (102) -> kept at original offset 2
  // Position 4: both image1/image2 transparent -> use image3's color1 (301) at
  // offset 6 -> 1+6=7
  IndexedPixelData expectedPixels = {4, 1, 5, 2, 7};

  REQUIRE(result.pixelData == expectedPixels);
  REQUIRE(result.palette == expectedPalette);
}