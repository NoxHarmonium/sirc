#ifndef QUANTIZER_H
#define QUANTIZER_H

#include "sircimage.hpp"

enum class PaletteReductionBpp : std::uint16_t {
  None = MAX_PALETTE_SIZE,
  FourBpp = 1 << 4,
  TwoBpp = 1 << 2
};

class Quantizer {
public:
  Quantizer() = default;

  [[nodiscard]] virtual SircImage quantize(const SircImage &sircImage,
                                           PaletteReductionBpp bpp) const = 0;

  Quantizer(const Quantizer &) = default;
  Quantizer &operator=(const Quantizer &) = default;
  Quantizer(Quantizer &&) noexcept = default;
  Quantizer &operator=(Quantizer &&) noexcept = default;
  virtual ~Quantizer() = default;
};

#endif // QUANTIZER_H
