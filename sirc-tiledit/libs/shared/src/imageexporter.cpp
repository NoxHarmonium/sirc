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
  // TODO: It doesn't look like the PPU has a way to set BPP yet?
  // Palette select is 3 bits so there can only be 8 palettes at the moment
  // The palette storage is 256 colours so that means we can only reference 8
  // palettes of 32 colours. That is a bit limiting I suppose. We might want
  // have more palettes with less colours in them.
  // If we can offset the palette index, we can use smaller palettes i.e. we
  // don't need to use all 3 bits of the select, and we can overlap palettes I
  // think we need to implement some sort of palette offset in the PPU registers

  uint16_t currentPaletteIndex = 0;
  uint16_t currentPaletteOffset = 0;
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
        "Palette {} (offset {})", currentPaletteIndex, currentPaletteOffset)));
    const libsirc::CPalette cPalette = {.comment =
                                            stringStorage.back()->c_str(),
                                        .data = palette->data(),
                                        .data_len = palette->size()};

    palettes[currentPaletteIndex] = cPalette;

    for (const auto &[name, sircImage] : images) {
      const auto &pixelData = sircImage.pixelData;
      // Create persistent storage for pixel data
      allPixelData.push_back(std::make_unique<std::vector<uint16_t>>(
          packIntVector<uint16_t, const size_t>(std::span(pixelData), 4)));
      const auto *const pixelData16 = allPixelData.back().get();

      stringStorage.push_back(std::make_unique<std::string>(
          std::format("tilemap_{}", currentPaletteIndex)));
      auto *const tileLabel = stringStorage.back()->c_str();

      stringStorage.push_back(
          std::make_unique<std::string>(std::format("Tilemap for {}", name)));
      auto *const tileComment = stringStorage.back()->c_str();

      // Create and store the tilemap
      tileMapStorage.push_back(
          libsirc::CTilemap{.label = tileLabel,
                            .comment = tileComment,
                            .palette_index = currentPaletteIndex,
                            .packed_pixel_data = pixelData16->data(),
                            .packed_pixel_data_len = pixelData16->size()});
    }

    currentPaletteOffset += palette->size();
    currentPaletteIndex++;
  }

  // Create the export structure
  stringStorage.push_back(std::make_unique<std::string>("palettes"));
  const auto *const paletteLabel = stringStorage.back()->c_str();
  libsirc::CTilemapExport export_data = {.tilemaps = tileMapStorage.data(),
                                         .tilemaps_len = tileMapStorage.size(),
                                         .palette_label = paletteLabel,
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