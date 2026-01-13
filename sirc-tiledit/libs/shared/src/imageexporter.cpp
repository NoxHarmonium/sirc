//
// Created by Sean Dawson on 10/8/2025.
//

#include "imageexporter.hpp"

#include "imagetileslicer.hpp"
#include "sircimage.hpp"

#include "utils.hpp"

#include <libsirc/libsirc.h>

std::string ImageExporter::exportToAsm(
    const std::unordered_map<SircPalette,
                             std::vector<std::pair<std::string, SircImage>>>
        &quantizedImagesByPalette) {
  // Important: These must be declared BEFORE tilemaps and palettes to avoid
  // dangling pointers
  // TODO: Hardcoded 16 values for 8x8 tiles (64 pixels / 4 bpp)
  // Will need to be 64 for 16x16 tiles
  std::vector<uint16_t>
      tileDataStorage; // Store vectors to keep pixel data alive
  std::vector<libsirc::CTilemap>
      tileMapStorage; // Store our actual CTilemap structs
  std::vector<std::unique_ptr<std::string>>
      stringStorage; // Store strings to keep them alive

  uint16_t currentPaletteIndex = 0;
  std::vector<libsirc::CPalette> palettes;
  const auto maxPaletteCount = quantizedImagesByPalette.size();

  for (const auto &[palette, images] : quantizedImagesByPalette) {
    if (currentPaletteIndex >= maxPaletteCount) {
      throw std::runtime_error(
          std::format("Palette index {} cannot fit into palette storage (256 "
                      "entries, or {} palettes)",
                      currentPaletteIndex, maxPaletteCount));
    }

    stringStorage.push_back(std::make_unique<std::string>(std::format(
        "palette__{}_{}", currentPaletteIndex, tileMapStorage.size())));
    auto *const paletteLabel = stringStorage.back()->c_str();

    stringStorage.push_back(std::make_unique<std::string>(
        std::format("Palette {} (number of values: {})", currentPaletteIndex,
                    palette->size())));
    auto *const paletteComment = stringStorage.back()->c_str();

    const libsirc::CPalette cPalette = {.label = paletteLabel,
                                        .comment = paletteComment,
                                        .data = palette->data(),
                                        .data_len = palette->size()};
    // TODO: Pad palette to 16 pixels?
    palettes.push_back(cPalette);

    for (const auto &[name, sircImage] : images) {
      auto [tileMapWithHashes, uniqueTiles] =
          ImageTileSlicer::slice(sircImage, TileSize::EightByEight);

      stringStorage.push_back(std::make_unique<std::string>(std::format(
          "tilemap__{}_{}", currentPaletteIndex, tileMapStorage.size())));
      auto *const tileLabel = stringStorage.back()->c_str();

      stringStorage.push_back(std::make_unique<std::string>(
          std::format("Tilemap for {} (number of tiles: {} (unique: {})", name,
                      tileMapWithHashes.size(), uniqueTiles.size())));
      auto *const tileComment = stringStorage.back()->c_str();

      std::unordered_map<TileReference, uint16_t> hashToIndex;
      auto tileIndex = 0u;
      for (auto &[tileHash, tileData] : uniqueTiles) {
        tileDataStorage.insert(std::end(tileDataStorage), std::begin(tileData),
                               std::end(tileData));
        hashToIndex[tileHash] = tileIndex;
        ++tileIndex;
      }

      auto tileMap = libsirc::CTilemap{
          .label = tileLabel,
          .comment = tileComment,
          .data = {},
      };

      // TODO: Rewrite with std::transform?
      // TODO: Assert bounds?
      for (size_t i = 0; i < tileMapWithHashes.size(); ++i) {
        tileMap.data[i] = hashToIndex.at(tileMapWithHashes[i]);
      }
      // Create and store the tilemap
      tileMapStorage.push_back(tileMap);
    }
  }

  // Create the export structure
  stringStorage.push_back(std::make_unique<std::string>("Tilesets Section"));
  const auto *const tilesetsComment = stringStorage.back()->c_str();
  stringStorage.push_back(std::make_unique<std::string>("Tilemaps Section"));
  const auto *const tilemapsComment = stringStorage.back()->c_str();
  stringStorage.push_back(std::make_unique<std::string>("Palettes Section"));
  const auto *const palettesComment = stringStorage.back()->c_str();

  // Only one tileset for now (can store 1024 tiles which is probably enough for
  // anyone)
  stringStorage.push_back(std::make_unique<std::string>("tileset_1"));
  const auto *const tilesetLabel = stringStorage.back()->c_str();
  stringStorage.push_back(std::make_unique<std::string>(std::format(
      "Tileset 1 (number of values: {}) ", tileDataStorage.size())));
  const auto *const tilesetComment = stringStorage.back()->c_str();
  auto tileSet = libsirc::CTileSet{.label = tilesetLabel,
                                   .comment = tilesetComment,
                                   .data = tileDataStorage.data(),
                                   .data_len = tileDataStorage.size()};
  auto tileSets = std::vector{tileSet};

  libsirc::CTilemapExport export_data = {.tilesets_comment = tilesetsComment,
                                         .tilesets = tileSets.data(),
                                         .tilesets_len = tileSets.size(),
                                         .tilemaps_comment = tilemapsComment,
                                         .tilemaps = tileMapStorage.data(),
                                         .tilemaps_len = tileMapStorage.size(),
                                         .palettes_comment = palettesComment,
                                         .palettes = palettes.data(),
                                         .palettes_len = palettes.size()};

  char *asmChar = libsirc::tilemap_to_str(export_data);

  std::string asmOutputStr;
  if (asmChar != nullptr) {
    asmOutputStr = asmChar;
    libsirc::free_str(asmChar);
  } else {
    throw std::runtime_error("Failed to generate assembly code");
  }

  return asmOutputStr;
}