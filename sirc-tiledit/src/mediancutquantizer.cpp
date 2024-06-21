#include "mediancutquantizer.h"

MedianCutQuantizer::MedianCutQuantizer() {}

SircImage MedianCutQuantizer::quantize(const SircImage &sircImage,
                                       const PaletteReductionBpp bpp) const {
  if (bpp == PaletteReductionBpp::None) {
    return sircImage;
  }

  // TODO: Quantize
  return sircImage;
}
