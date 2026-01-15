//
// Created by Sean Dawson on 11/1/2026.
//

#include "imagetileslicer.hpp"

#include "utils.hpp"
#include <iostream>
#include <span>

[[nodiscard]] std::pair<
    TileMapData, std::unordered_map<TileReference, std::vector<uint16_t>>>
ImageTileSlicer::slice(const SircImage &inputImage, const TileSize tileSize) {
  if (tileSize == TileSize::SixteenBySixteen) {
    // We would need a bigger image to fit 256x256 pixels into a 32x32 tilemap
    // Since the tilemap size is fixed, if we want to use 16x16 tiles, the image
    // would have to be 512x512
    throw std::runtime_error("Sixteen by sixteen tiles not supported yet");
  }
  // Update this when support 16x16 tiles

  std::unordered_map<TileReference, std::vector<uint16_t>> uniqueTiles;
  TileMapData tileMapData{};

  for (int ty = 0; ty < HEIGHT_TILEMAP; ++ty) {
    for (int tx = 0; tx < WIDTH_TILEMAP; ++tx) {
      const int dimension = 8;

      std::vector<PaletteReference> unpackedTilePixels;
      const auto totalUnpackedPixels = dimension * dimension;
      unpackedTilePixels.reserve(totalUnpackedPixels);

      for (int py = 0; py < dimension; ++py) {
        for (int px = 0; px < dimension; ++px) {
          const int x = tx * dimension + px;
          const int y = ty * dimension + py;

          unpackedTilePixels.push_back(
              inputImage.pixelData[y * WIDTH_PIXELS + x]);
        }
      }

      assert(unpackedTilePixels.size() == totalUnpackedPixels);

      std::vector<uint16_t> packedTilePixels =
          packIntVector<uint16_t, PaletteReference>(
              std::span(unpackedTilePixels), 4);

      assert(packedTilePixels.size() == totalUnpackedPixels / 4);
      auto hash = hash_vector(std::span(packedTilePixels));
      uniqueTiles[hash] = packedTilePixels;
      tileMapData[ty * WIDTH_TILEMAP + tx] = hash;
    }
  }

  return {tileMapData, uniqueTiles};
}