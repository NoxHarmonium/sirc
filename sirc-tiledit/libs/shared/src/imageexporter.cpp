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
  std::vector<libsirc::CTilemap>
      tileMapStorage; // Store our actual CTilemap structs
  std::vector<std::unique_ptr<std::string>>
      stringStorage; // Store strings to keep them alive

  uint16_t currentPaletteIndex = 0;
  // Currently all tilemaps share the same tileset base address
  // so this index has to be in the outer scope
  uint16_t currentTileIndex = 0;
  std::vector<libsirc::CPalette> palettes;
  std::vector<uint16_t> tileSetData;
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
      hashToIndex.reserve(uniqueTiles.size());
      for (auto &[tileHash, tileData] : uniqueTiles) {
        // Insert full 8x8 tile into storage
        tileSetData.insert(std::end(tileSetData), std::begin(tileData),
                           std::end(tileData));
        hashToIndex[tileHash] = currentTileIndex;
        ++currentTileIndex;
        assert(currentTileIndex <
               0x3FF); // Tile index is stored in 10 bits - any larger and we
                       // need to have different tilesets per bg layer
      }

      auto tileMap = libsirc::CTilemap{
          .label = tileLabel,
          .comment = tileComment,
          .data = {},
      };

      assert(tileMapWithHashes.size() == libsirc::TILEMAP_SIZE);
      const auto tilemapDataSpan =
          std::span(tileMap.data, libsirc::TILEMAP_SIZE);
      std::ranges::transform(tileMapWithHashes, tilemapDataSpan.begin(),
                             [hashToIndex](const size_t tileHash) -> uint16_t {
                               return hashToIndex.at(tileHash);
                             });

      // Create and store the tilemap
      tileMapStorage.push_back(tileMap);
    }

    ++currentPaletteIndex;
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
  stringStorage.push_back(std::make_unique<std::string>("tileset_0"));
  const auto *const tilesetLabel = stringStorage.back()->c_str();
  stringStorage.push_back(std::make_unique<std::string>(
      std::format("Tileset 0 (number of tiles: {}, number of values: {}) ",
                  tileSetData.size() / 16, tileSetData.size())));
  const auto *const tilesetComment = stringStorage.back()->c_str();
  auto tileSet = libsirc::CTileSet{.label = tilesetLabel,
                                   .comment = tilesetComment,
                                   .data = tileSetData.data(),
                                   .data_len = tileSetData.size()};
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