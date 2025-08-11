//
// Created by Sean Dawson on 10/8/2025.
//

#include "imageexporter.hpp"

#include "utils.hpp"

#include <libsirc/libsirc.h>

std::string ImageExporter::exportToAsm(
    const std::unordered_map<SircPalette, std::vector<SircImage>>
        &quantizedImagesByPalette,
    const uint bpp) {
  // Important: These must be declared BEFORE tilemaps and palettes to avoid
  // dangling pointers
  std::vector<std::vector<uint16_t>>
      allPixelData; // Store vectors to keep pixel data alive
  std::vector<libsirc::CTilemap>
      tileMapStorage; // Store our actual CTilemap structs
  const uint paletteSize = 1 << bpp;
  const uint maxPaletteCount = MAX_PALETTE_SIZE / paletteSize;

  uint16_t currentPaletteIndex = 0;
  std::vector<libsirc::CPalette> palettes;

  static auto defaultPaletteComment = "";
  const libsirc::CPalette defaultPalette = {.comment = defaultPaletteComment,
                                            .data = {}};
  palettes.resize(maxPaletteCount, defaultPalette);

  for (const auto &[palette, images] : quantizedImagesByPalette) {
    if (currentPaletteIndex >= maxPaletteCount) {
      throw std::runtime_error(
          std::format("Palette index {} cannot fit into palette storage (256 "
                      "entries, or {} palettes)",
                      currentPaletteIndex, maxPaletteCount));
    }

    static const char *paletteComment = "palette comment";
    libsirc::CPalette cPalette = {.comment = paletteComment, .data = {}};
    if (palette->size() > paletteSize) {
      throw std::runtime_error(std::format(
          "Provided palette has {} colors, but only {} colors can fit in "
          "{} bpp",
          palette->size(), paletteSize, bpp));
    }

    std::copy_n(palette->begin(),
                std::min(palette->size(), static_cast<size_t>(paletteSize)),
                cPalette.data);

    palettes[currentPaletteIndex] = cPalette;

    for (const auto &[_, pixelData] : images) {
      // Create persistent storage for pixel data
      allPixelData.emplace_back(
          packIntVector<uint16_t, const size_t>(std::span(pixelData), 4));
      const auto &pixelData16 = allPixelData.back();

      // Static strings for labels/comments
      static const char *tileLabel = "some_label";
      static const char *tileComment = "some_comment";

      // Create and store the tilemap
      tileMapStorage.push_back(
          libsirc::CTilemap{.label = tileLabel,
                            .comment = tileComment,
                            .palette_index = currentPaletteIndex,
                            .packed_pixel_data = pixelData16.data(),
                            .packed_pixel_data_len = pixelData16.size()});
    }

    currentPaletteIndex++;
  }

  // Create the export structure
  static const char *palLabel = "some_label";
  libsirc::CTilemapExport export_data = {.tilemaps = tileMapStorage.data(),
                                         .tilemaps_len = tileMapStorage.size(),
                                         .palette_label = palLabel,
                                         .palettes = {}};

  // Copy palettes into the export structure
  std::copy_n(
      palettes.begin(),
      std::min(currentPaletteIndex, static_cast<uint16_t>(maxPaletteCount)),
      export_data.palettes);

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