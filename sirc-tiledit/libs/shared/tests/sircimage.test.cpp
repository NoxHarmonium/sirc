#include <miscadapter.hpp>
#include <vector>

#include <catch2/catch_test_macros.hpp>

#include <sircimage.hpp>

TEST_CASE("Image data is untouched", "[fromSircImageData]") {
  const SircImageData imageData = {.palette = {1, 2, 3},
                                   .pixelData = {{{0, 1, 0, 2, 1}}}};
  const auto sircImage = MiscAdapter::fromSircImageData(imageData);

  REQUIRE(imageData == sircImage.getImageData());
}

TEST_CASE("Pixel data is converted to indexed format", "[fromPixelData]") {
  // Note: ensure that the first pixel is zero, so that the first palette entry
  // is zero and we don't need to pad out the expected pixel data with whatever
  // the index is mapped to zero
  const PackedPixelData inputPixelData = {{{0x0, 0xA, 0xB, 0xA, 0xC, 0xB}}};
  const auto sircImage = MiscAdapter::fromPixelData(inputPixelData);
  const auto [palette, outputPixelData] = sircImage.getImageData();

  REQUIRE(IndexedPixelData{{{0, 1, 2, 1, 3, 2}}} == outputPixelData);
  REQUIRE(std::vector<SircColor>{0x0, 0xA, 0xB, 0xC} == palette);
}