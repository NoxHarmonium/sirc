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

  auto const asmOutput =
      ImageExporter::exportToAsm(quantizedImagesByPalette).substr(0, 98);

  std::cout << asmOutput << "\n";

  // TODO: A better test
  REQUIRE(asmOutput.starts_with(
      ";Tilesets Section\n;Tileset 1\n:tileset_1\n.DW #0x6666\n.DW "
      "#0x7000\n.DW #0x0000\n.DW #0x0000\n.DW #0x000"));
}
