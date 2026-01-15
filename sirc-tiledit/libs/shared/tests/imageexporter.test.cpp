#include "imageexporter.hpp"

#include <vector>

#include "catch2/catch_amalgamated.hpp"

#include <iostream>
#include <ranges>
#include <sircimage.hpp>

TEST_CASE("Exports images correctly") {
  const auto palette1 =
      std::make_shared<std::vector<SircColor>>(std::initializer_list<SircColor>{
          100, 101, 102, 0, 201, 202, 0, 301, 302});
  const auto palette2 =
      std::make_shared<std::vector<SircColor>>(std::initializer_list<SircColor>{
          200, 201, 202, 0, 301, 302, 0, 401, 402});

  const SircImage sircImage1 = {.palette = palette1,
                                .pixelData = {0, 1, 0, 2, 0}};
  const SircImage sircImage2 = {.palette = palette1,
                                .pixelData = {4, 3, 5, 3, 3}};
  const SircImage sircImage3 = {.palette = palette2,
                                .pixelData = {6, 6, 6, 6, 7}};

  const std::unordered_map<SircPalette,
                           std::vector<std::pair<std::string, SircImage>>>
      quantizedImagesByPalette = {
          {palette1, {{"sircImage1", sircImage1}, {"sircImage2", sircImage2}}},
          {palette2, {{"sircImage3", sircImage3}}}};

  auto const asmOutput = ImageExporter::exportToAsm(quantizedImagesByPalette);

  std::cout << asmOutput << "\n";

  const auto expectedLines =
      std::array{";Tilesets Section",
                 ";Tileset 0 (number of tiles: 6, number of values: 96) ",
                 ":tileset_0",
                 ";Tilemaps Section",
                 ";Tilemap for sircImage3 (number of tiles: 1024 (unique: 2)",
                 ":tilemap__0_0",
                 ";Tilemap for sircImage1 (number of tiles: 1024 (unique: 2)",
                 ":tilemap__1_1",
                 ";Tilemap for sircImage2 (number of tiles: 1024 (unique: 2)",
                 ":tilemap__1_2",
                 ";Palettes Section",
                 ";Palette 0 (number of values: 9)",
                 ":palette__0_0",
                 ";Palette 1 (number of values: 9)",
                 ":palette__1_1"};

  size_t lastPos = 0;
  for (const auto &line : expectedLines) {
    std::cout << "Checking for line: " << line << "\n";
    const auto pos = asmOutput.find(line);
    // Check for the presence of line in the output
    REQUIRE(pos != std::string::npos);
    // Enforce ordering of expected lines
    REQUIRE(pos >= lastPos);
    lastPos = pos;
  }
}
