#include <vector>

#include <catch2/catch_test_macros.hpp>

#include <sircimage.hpp>

TEST_CASE("Image data is untouched but palette map is populated",
          "[fromSircImageData]") {
  const SircImageData imageData = {.palette = {1, 2, 3},
                                   .pixelData = {{{0, 1, 0, 2, 1}}}};
  const auto sircImage = SircImage::fromSircImageData(imageData);

  REQUIRE(imageData == sircImage.getImageData());
  REQUIRE(sircImage.paletteIndexForColor(1) == 0);
  REQUIRE(sircImage.paletteIndexForColor(2) == 1);
  REQUIRE(sircImage.paletteIndexForColor(3) == 2);
}

TEST_CASE("Pixel data is converted to indexed format and palette is populated",
          "[fromPixelData]") {
  // Note: ensure that the first pixel is zero, so that the first palette entry
  // is zero and we don't need to pad out the expected pixel data with whatever
  // the index is mapped to zero
  const PackedPixelData inputPixelData = {{{0x0, 0xA, 0xB, 0xA, 0xC, 0xB}}};
  const auto sircImage = SircImage::fromPixelData(inputPixelData);
  const auto [palette, outputPixelData] = sircImage.getImageData();

  REQUIRE(IndexedPixelData{{{0, 1, 2, 1, 3, 2}}} == outputPixelData);
  REQUIRE(std::vector<SircColor>{0x0, 0xA, 0xB, 0xC} == palette);
  REQUIRE(sircImage.paletteIndexForColor(0x0) == 0);
  REQUIRE(sircImage.paletteIndexForColor(0xA) == 1);
  REQUIRE(sircImage.paletteIndexForColor(0xB) == 2);
  REQUIRE(sircImage.paletteIndexForColor(0xC) == 3);
}