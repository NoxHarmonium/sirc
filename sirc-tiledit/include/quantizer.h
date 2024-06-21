#ifndef QUANTIZER_H
#define QUANTIZER_H

#include "sircimage.h"

enum class PaletteReductionBpp { None, FourBpp, TwoBpp };

class Quantizer {
public:
  virtual SircImage quantize(const SircImage &sircImage,
                             const PaletteReductionBpp bpp) const = 0;
};

#endif // QUANTIZER_H
