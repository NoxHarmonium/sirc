#include <algorithm>
#include <vector>

#include <catch2/catch_test_macros.hpp>

#include <mediancutquantizer.hpp>
#include <sircimage.hpp>

TEST_CASE("Reduces palette size to 2bpp", "[quantize]") {
  const SircImageData imageData = {
      .pixelData = {{{0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10}}},
      .palette = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11}};
  const auto sircImage = SircImage::fromSircImageData(imageData);

  const auto quantizer = MedianCutQuantizer();
  const auto quantizedImage =
      quantizer.quantize(sircImage, PaletteReductionBpp::TwoBpp);
  const auto [palette, pixelData] = quantizedImage.getImageData();

  REQUIRE(4 == palette.size());
  REQUIRE(std::vector<SircColor>{1, 4, 7, 10} == palette);
  // TODO: Think of a better way to iterate a 2D array
  // category=Refactoring
  REQUIRE(std::all_of(
      pixelData.cbegin(), pixelData.cend(),
      [&palette](const std::array<PaletteReference, HEIGHT_PIXELS> row) {
        return std::all_of(row.cbegin(), row.cend(),
                           [&palette](const PaletteReference pixel) {
                             return pixel < palette.size();
                           });
      }));
}