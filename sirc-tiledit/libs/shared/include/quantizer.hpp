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

  /**
   * Takes an indexed image with a palette and reduces the palette so it can be
   * referenced by pixel values with the given bpp.
   * E.g. 4bpp = 16 colours addressable -> therefore, max 16 colours
   * @param sircImage the image to quantise the palette for
   * @param bpp determines the max size of the palette
   * @return a copy of the image with the palette remapped
   */
  [[nodiscard]] virtual SircImage quantize(const SircImage &sircImage,
                                           PaletteReductionBpp bpp) const = 0;
  /**
   * Takes indexed images with palettes, combines their palettes and reduces the
   * combined palette so it can be referenced by pixel values with the given
   * bpp. E.g. 4bpp = 16 colours addressable -> therefore, max 16 colours
   *
   * Useful when you want different tilemaps to share the same palette.
   *
   * @param sircImages the images to quantise the palette for
   * @param bpp determines the max size of the palette
   * @return a copy of the image with the palette remapped
   */
  [[nodiscard]] virtual std::vector<SircImage>
  quantize_all(const std::vector<SircImage> &sircImages,
               PaletteReductionBpp bpp) const = 0;

  Quantizer(const Quantizer &) = default;
  Quantizer &operator=(const Quantizer &) = default;
  Quantizer(Quantizer &&) noexcept = default;
  Quantizer &operator=(Quantizer &&) noexcept = default;
  virtual ~Quantizer() = default;
};

#endif // QUANTIZER_H
