//
// Created by Sean Dawson on 10/8/2025.
//

#include "imageexporter.hpp"
#include "sircimage.hpp"

#include "utils.hpp"

#include <libsirc/libsirc.h>

std::string ImageExporter::exportToAsm(
    const std::unordered_map<SircPalette,
                             std::vector<std::pair<std::string, SircImage>>>
        &quantizedImagesByPalette) {
  // Important: These must be declared BEFORE tilemaps and palettes to avoid
  // dangling pointers
  std::vector<std::unique_ptr<std::vector<uint16_t>>>
      allPixelData; // Store vectors to keep pixel data alive
  std::vector<libsirc::CTilemap>
      tileMapStorage; // Store our actual CTilemap structs
  std::vector<std::unique_ptr<std::string>>
      stringStorage; // Store strings to keep them alive

  // TODO: WAIT A MINUTE
  // Is this exporting raw pixel data, not tiles?
  // :FACEPALM:
  // TODO: Chop up into tiles

  uint16_t currentPaletteIndex = 0;
  std::vector<libsirc::CPalette> palettes;
  const auto maxPaletteCount = quantizedImagesByPalette.size();

  static const auto *defaultPaletteComment = "";
  const libsirc::CPalette defaultPalette = {
      .comment = defaultPaletteComment, .data = nullptr, .data_len = 0};
  palettes.resize(maxPaletteCount, defaultPalette);

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

    palettes[currentPaletteIndex] = cPalette;

    for (const auto &[name, sircImage] : images) {
      const auto &pixelData = sircImage.pixelData;
      // Create persistent storage for pixel data
      allPixelData.push_back(std::make_unique<std::vector<uint16_t>>(
          packIntVector<uint16_t, const size_t>(std::span(pixelData), 4)));
      const auto *const pixelData16 = allPixelData.back().get();

      stringStorage.push_back(std::make_unique<std::string>(std::format(
          "tilemap__{}_{}", currentPaletteIndex, tileMapStorage.size())));
      auto *const tileLabel = stringStorage.back()->c_str();

      stringStorage.push_back(std::make_unique<std::string>(
          std::format("Tilemap for {} (number of packed 16-bit values: {})",
                      name, pixelData16->size())));
      auto *const tileComment = stringStorage.back()->c_str();

      // Create and store the tilemap
      tileMapStorage.push_back(
          libsirc::CTilemap{.label = tileLabel,
                            .comment = tileComment,
                            .data = pixelData16->data(),
                            .data_len = pixelData16->size()});
    }

    currentPaletteIndex++;
  }

  // Create the export structure
  stringStorage.push_back(std::make_unique<std::string>("Tileset Section"));
  const auto *const tilesetsComment = stringStorage.back()->c_str();
  stringStorage.push_back(std::make_unique<std::string>("Tilemap Section"));
  const auto *const tilemapsComment = stringStorage.back()->c_str();
  stringStorage.push_back(std::make_unique<std::string>("Palette Section"));
  const auto *const palettesComment = stringStorage.back()->c_str();
  libsirc::CTilemapExport export_data = {.tilesets_comment = tilesetsComment,
                                         // TODO: Implement tile generation
                                         .tilesets = nullptr,
                                         .tilesets_len = 0,
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