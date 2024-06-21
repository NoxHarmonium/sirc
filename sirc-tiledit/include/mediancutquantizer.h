#ifndef MEDIANCUTQUANTIZER_H
#define MEDIANCUTQUANTIZER_H

#include "quantizer.h"

/**
 * @brief A simple quantizer that can only reduce the palette in multiples of
 * two
 *
 * @see
 * https://gowtham000.hashnode.dev/median-cut-a-popular-colour-quantization-strategy
 */
class MedianCutQuantizer : public Quantizer {
public:
  MedianCutQuantizer() = default;

  [[nodiscard]] SircImage
  quantize(const SircImage &sircImage,
           const PaletteReductionBpp bpp) const override;
};

#endif // MEDIANCUTQUANTIZER_H
