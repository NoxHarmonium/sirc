//
// Created by Sean Dawson on 10/8/2025.
//

#include "imageexporter.hpp"

#include "utils.hpp"

#include <libsirc/libsirc.h>

std::string ImageExporter::exportToAsm(
    const std::unordered_map<SircPalette, std::vector<SircImage>>
        &quantizedImagesByPalette) {
  // Important: These must be declared BEFORE tilemaps and palettes to avoid
  // dangling pointers
  std::vector<std::vector<uint16_t>>
      allPixelData; // Store vectors to keep pixel data alive
  std::vector<libsirc::CTilemap>
      tileMapStorage; // Store our actual CTilemap structs

  // Create a C-compatible array of CTilemap pointers that will be passed to the
  // C function
  uint16_t paletteIndex = 0;
  std::array<libsirc::CPalette, 16> palettes{};

  static const char *defaultPaletteComment = "";
  libsirc::CPalette defaultPalette = {.comment = defaultPaletteComment,
                                      .data = {}};
  for (int i = 0; i < 16; i++) {
    palettes[i] = defaultPalette;
  }

  for (const auto &[palette, images] : quantizedImagesByPalette) {
    if (paletteIndex >= 16) {
      throw std::runtime_error("Too many palettes (max 16)");
    }

    static const char *paletteComment = "palette comment";
    libsirc::CPalette cPalette = {.comment = paletteComment, .data = {}};
    if (palette->size() > 16) {
      throw std::runtime_error("Palette has more than 16 colors");
    }

    std::copy_n(palette->begin(),
                std::min(palette->size(), static_cast<size_t>(16)),
                cPalette.data);

    palettes[paletteIndex] = cPalette;

    for (const auto &[_, pixelData] : images) {
      // Create persistent storage for pixel data
      allPixelData.emplace_back(
          safeCastIntVector<uint16_t, const size_t>(std::span(pixelData)));
      const auto &pixelData16 = allPixelData.back();

      // Static strings for labels/comments
      static const char *tileLabel = "some_label";
      static const char *tileComment = "some_comment";

      // Create and store the tilemap
      tileMapStorage.push_back(
          libsirc::CTilemap{.label = tileLabel,
                            .comment = tileComment,
                            .palette_index = paletteIndex,
                            .packed_pixel_data = pixelData16.data(),
                            .packed_pixel_data_len = pixelData16.size()});
    }

    paletteIndex++;
  }

  // Create the export structure
  static const char *palLabel = "some_label";
  libsirc::CTilemapExport export_data = {.tilemaps = tileMapStorage.data(),
                                         .tilemaps_len = tileMapStorage.size(),
                                         .palette_label = palLabel,
                                         .palettes = {}};

  // Copy palettes into the export structure
  std::copy_n(palettes.begin(),
              std::min(paletteIndex, static_cast<uint16_t>(16)),
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