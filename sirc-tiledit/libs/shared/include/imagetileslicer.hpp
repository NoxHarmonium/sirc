#ifndef SIRC_TILEDIT_IMAGETILESLICER_HPP
#define SIRC_TILEDIT_IMAGETILESLICER_HPP
#include "sircimage.hpp"

#include <unordered_map>
#include <utility>
#include <vector>

using TileReference = size_t;
using TileMapData = std::array<TileReference, TOTAL_TILES>;

enum class TileSize {
  // 4 bpp => 4 pixels per 16 bit value
  // 8*8=64 pixels => 64/4 = 16 x 16 bit values
  EightByEight = 16,
  // 16*16=256 pixels => 256/4 = 64 x 16 bit values
  SixteenBySixteen = 64
};

class ImageTileSlicer {

public:
  ImageTileSlicer() = default;

  // Tilemaps are 32x32 - images are 512x512
  // Slice up image into tiles and deduplcate them
  [[nodiscard]] static std::pair<
      TileMapData, std::unordered_map<TileReference, std::vector<uint16_t>>>
  slice(const SircImage &inputImage, TileSize tileSize);
};

#endif // SIRC_TILEDIT_IMAGETILESLICER_HPP
