#include <catch2/catch_test_macros.hpp>

#include <sircimage.hpp>

TEST_CASE("Image data is untouched but palette map is populated",
          "[fromSircImageData]") {
  SircImageData imageData = {.pixelData = {{{0, 1, 0, 2, 1}}},
                             .palette = {1, 2, 3}};
  auto sircImage = SircImage::fromSircImageData(imageData);

  REQUIRE(imageData == sircImage.getImageData());
  REQUIRE(sircImage.paletteIndexForColor(1) == 0);
  REQUIRE(sircImage.paletteIndexForColor(2) == 1);
  REQUIRE(sircImage.paletteIndexForColor(3) == 2);
}