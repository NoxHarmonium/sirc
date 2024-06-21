#ifndef MEDIANCUTQUANTIZER_H
#define MEDIANCUTQUANTIZER_H

#include "quantizer.h"

class MedianCutQuantizer : public Quantizer {
public:
  MedianCutQuantizer();

  SircImage quantize(const SircImage &sircImage,
                     const PaletteReductionBpp bpp) const override;
};

#endif // MEDIANCUTQUANTIZER_H
