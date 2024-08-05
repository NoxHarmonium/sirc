#include "nanobench/nanobench.h"

#include <filesystem>
#include <fstream>

#include "testconfig.h"
#include <imageloader.hpp>
#include <mediancutquantizer.hpp>
#include <rgbaadapter.hpp>
#include <sircimage.hpp>

SircImage setupBenchmark(const std::filesystem::path &inputPath) {
  const std::filesystem::path testRootPath(BENCHMARK_ROOT);

  const auto inputPixelData =
      ImageLoader::loadImageFromPng((testRootPath / inputPath).c_str());
  return RgbaAdapter::rgbaToSircImage(inputPixelData);
}

int main() {
  const auto pixelArtBackgroundImage = setupBenchmark(
      std::filesystem::path("resources/pixel_art_background.png"));
  const auto gradientImage =
      setupBenchmark(std::filesystem::path("resources/gradient.png"));
  const auto redfloweringGumImage =
      setupBenchmark(std::filesystem::path("resources/red_flowering_gum.png"));
  const auto quantizer = MedianCutQuantizer();
  constexpr int epochs = 500;

  std::ofstream pixelArtBackground2BppPyPerf("pixel_art_background-2bpp.json");
  std::ofstream pixelArtBackground4BppPyPerf("pixel_art_background-4bpp.json");
  std::ofstream gradient2BppPyPerf("gradient-2bpp.json");
  std::ofstream gradient4BppPyPerf("gradient-4bpp.json");
  std::ofstream redFloweringGum2BppPyPerf("red_flowering_gum-2bpp.json");
  std::ofstream redFloweringGum4BppPyPerf("red_flowering_gum-4bpp.json");
  std::ofstream redFloweringGum8BppPyPerf("red_flowering_gum-8bpp.json");

  ankerl::nanobench::Bench()
      .epochs(epochs)
      .run("Quantizes a real test image correctly (pixel_art_background/2bpp)",
           [&] {
             ankerl::nanobench::doNotOptimizeAway(quantizer.quantize(
                 pixelArtBackgroundImage, PaletteReductionBpp::TwoBpp));
           })
      .render(ankerl::nanobench::templates::pyperf(),
              pixelArtBackground2BppPyPerf);

  ankerl::nanobench::Bench()
      .epochs(epochs)
      .run("Quantizes a real test image correctly (pixel_art_background/4bpp)",
           [&] {
             ankerl::nanobench::doNotOptimizeAway(quantizer.quantize(
                 pixelArtBackgroundImage, PaletteReductionBpp::FourBpp));
           })
      .render(ankerl::nanobench::templates::pyperf(),
              pixelArtBackground4BppPyPerf);

  ankerl::nanobench::Bench()
      .epochs(epochs)
      .run("Quantizes a real test image correctly (gradient/2bpp)",
           [&] {
             ankerl::nanobench::doNotOptimizeAway(quantizer.quantize(
                 gradientImage, PaletteReductionBpp::TwoBpp));
           })
      .render(ankerl::nanobench::templates::pyperf(), gradient2BppPyPerf);

  ankerl::nanobench::Bench()
      .epochs(epochs)
      .run("Quantizes a real test image correctly (gradient/4bpp)",
           [&] {
             ankerl::nanobench::doNotOptimizeAway(quantizer.quantize(
                 gradientImage, PaletteReductionBpp::FourBpp));
           })
      .render(ankerl::nanobench::templates::pyperf(), gradient4BppPyPerf);

  ankerl::nanobench::Bench()
      .epochs(epochs)
      .run("Quantizes a real test image correctly (red_flowering_gum/2bpp)",
           [&] {
             ankerl::nanobench::doNotOptimizeAway(quantizer.quantize(
                 redfloweringGumImage, PaletteReductionBpp::TwoBpp));
           })
      .render(ankerl::nanobench::templates::pyperf(),
              redFloweringGum2BppPyPerf);

  ankerl::nanobench::Bench()
      .epochs(epochs)
      .run("Quantizes a real test image correctly (red_flowering_gum/4bpp)",
           [&] {
             ankerl::nanobench::doNotOptimizeAway(quantizer.quantize(
                 redfloweringGumImage, PaletteReductionBpp::FourBpp));
           })
      .render(ankerl::nanobench::templates::pyperf(),
              redFloweringGum4BppPyPerf);

  ankerl::nanobench::Bench()
      .epochs(epochs)
      .run("Quantizes a real test image correctly (red_flowering_gum/8bpp)",
           [&] {
             ankerl::nanobench::doNotOptimizeAway(quantizer.quantize(
                 redfloweringGumImage, PaletteReductionBpp::None));
           })
      .render(ankerl::nanobench::templates::pyperf(),
              redFloweringGum8BppPyPerf);
}