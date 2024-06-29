#include <algorithm>
#include <filesystem>
#include <vector>

#include <catch2/catch_test_macros.hpp>

#include "testconfig.h"
#include "utils.hpp"
#include <imageloader.hpp>
#include <mediancutquantizer.hpp>
#include <rgbaadapter.hpp>
#include <sircimage.hpp>

void runIntegrationTest(const std::filesystem::path &inputPath,
                        const std::filesystem::path &outputPath,
                        const std::filesystem::path &referencePath,
                        const PaletteReductionBpp &bpp) {
  const std::filesystem::path testRootPath(TEST_ROOT);
  const auto fullOutputPath = testRootPath / outputPath;
  const auto fullReferencePath = (testRootPath / referencePath);

  const auto inputPixelData =
      ImageLoader::loadImageFromPng((testRootPath / inputPath).c_str());
  const auto sircImage = RgbaAdapter::rgbaToSircImage(inputPixelData);
  const auto outputImageBeforeQuant = RgbaAdapter::sircImageToRgba(sircImage);

  auto beforeQuantPath = fullOutputPath.string();
  auto replaceMe = std::string("output");
  beforeQuantPath.replace(beforeQuantPath.find(replaceMe), replaceMe.length(),
                          "______");

  ImageLoader::saveImageToPng(beforeQuantPath.c_str(), outputImageBeforeQuant);

  const auto quantizer = MedianCutQuantizer();
  const auto quantizedImage = quantizer.quantize(sircImage, bpp);
  const auto outputImage = RgbaAdapter::sircImageToRgba(quantizedImage);

  // Save the data to a PNG for visual comparison when debugging
  ImageLoader::saveImageToPng(fullOutputPath.c_str(), outputImage);

  const auto referencePixelData =
      ImageLoader::loadImageFromPng(fullReferencePath.c_str());

  bool allPixelsMatch = true;
  for (size_t y = 0; y < HEIGHT_PIXELS; y++) {
    for (size_t x = 0; x < WIDTH_PIXELS; x++) {
      // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
      allPixelsMatch &= (referencePixelData[x][y] == outputImage[x][y]);
    }
  }
  REQUIRE(allPixelsMatch);
}

TEST_CASE("Quantizes a real test image correctly (pixel_art_background/2bpp)",
          "[integration]") {
  runIntegrationTest(
      std::filesystem::path("resources/pixel_art_background.png"),
      std::filesystem::path(
          "resources/pixel_art_background_output_actual_2bpp.png"),
      std::filesystem::path(
          "resources/pixel_art_background_output_expected_2bpp.png"),
      PaletteReductionBpp::TwoBpp);
}

TEST_CASE("Quantizes a real test image correctly (pixel_art_background/4bpp)",
          "[integration]") {
  runIntegrationTest(
      std::filesystem::path("resources/pixel_art_background.png"),
      std::filesystem::path(
          "resources/pixel_art_background_output_actual_4bpp.png"),
      std::filesystem::path(
          "resources/pixel_art_background_output_expected_4bpp.png"),
      PaletteReductionBpp::FourBpp);
}

TEST_CASE("Quantizes a real test image correctly (gradient/2bpp)",
          "[integration]") {
  runIntegrationTest(
      std::filesystem::path("resources/gradient.png"),
      std::filesystem::path("resources/gradient_output_actual_2bpp.png"),
      std::filesystem::path("resources/gradient_output_expected_2bpp.png"),
      PaletteReductionBpp::TwoBpp);
}

TEST_CASE("Quantizes a real test image correctly (gradient/4bpp)",
          "[integration]") {
  runIntegrationTest(
      std::filesystem::path("resources/gradient.png"),
      std::filesystem::path("resources/gradient_output_actual_4bpp.png"),
      std::filesystem::path("resources/gradient_output_expected_4bpp.png"),
      PaletteReductionBpp::FourBpp);
}

TEST_CASE("Quantizes a real test image correctly (red_flowering_gum/2bpp)",
          "[integration]") {
  runIntegrationTest(
      std::filesystem::path("resources/red_flowering_gum.png"),
      std::filesystem::path(
          "resources/red_flowering_gum_output_actual_2bpp.png"),
      std::filesystem::path(
          "resources/red_flowering_gum_output_expected_2bpp.png"),
      PaletteReductionBpp::TwoBpp);
}

TEST_CASE("Quantizes a real test image correctly (red_flowering_gum/4bpp)",
          "[integration]") {
  runIntegrationTest(
      std::filesystem::path("resources/red_flowering_gum.png"),
      std::filesystem::path(
          "resources/red_flowering_gum_output_actual_4bpp.png"),
      std::filesystem::path(
          "resources/red_flowering_gum_output_expected_4bpp.png"),
      PaletteReductionBpp::FourBpp);
}

TEST_CASE("Quantizes a real test image correctly (red_flowering_gum/256bpp)",
          "[integration]") {
  runIntegrationTest(
      std::filesystem::path("resources/red_flowering_gum.png"),
      std::filesystem::path(
          "resources/red_flowering_gum_output_actual_256bpp.png"),
      std::filesystem::path(
          "resources/red_flowering_gum_output_expected_256bpp.png"),
      PaletteReductionBpp::None);
}