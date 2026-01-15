#include "imagetileslicer.hpp"
#include "catch2/catch_amalgamated.hpp"
#include <vector>

TEST_CASE("ImageTileSlicer::slice - 8x8 tiles") {
  SircImage image = SircImage::empty();
  // Fill image with some data
  // 256x256 pixels
  for (size_t i = 0; i < TOTAL_PIXELS; ++i) {
    image.pixelData[i] = (i / WIDTH_PIXELS) % 16;
  }

  auto [tileMap, uniqueTiles] =
      ImageTileSlicer::slice(image, TileSize::EightByEight);

  // Every row is the same, and the palette value range is 0-15
  // So the first 8 values produces a tile with 0s-7s
  // The next 8 values produces tiles with 8s-15s
  // and then it repeats, so there are only two unique tiles.
  REQUIRE(uniqueTiles.size() == 2);
  REQUIRE(tileMap[0] == 12849195384201909820u);
  REQUIRE(tileMap[32] == 3924250986184893242u);
  REQUIRE(uniqueTiles[12849195384201909820u] ==
          std::vector<uint16_t>{0x0, 0x0, 0x1111, 0x1111, 0x2222, 0x2222,
                                0x3333, 0x3333, 0x4444, 0x4444, 0x5555, 0x5555,
                                0x6666, 0x6666, 0x7777, 0x7777});
  REQUIRE(uniqueTiles[3924250986184893242u] ==
          std::vector<uint16_t>{0x8888, 0x8888, 0x9999, 0x9999, 0xAAAA, 0xAAAA,
                                0xBBBB, 0xBBBB, 0xCCCC, 0xCCCC, 0xDDDD, 0xDDDD,
                                0xEEEE, 0xEEEE, 0xFFFF, 0xFFFF});
}
