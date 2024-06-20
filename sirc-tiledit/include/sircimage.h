#ifndef IMAGEPROCESSOR_H
#define IMAGEPROCESSOR_H

#include <QImage>
#include <QPixmap>
#include <array>
#include <map>
#include <vector>

using SircColor = u_int16_t;
using PaletteReference = u_int8_t;

const int WIDTH_PIXELS = 256;
const int HEIGHT_PIXELS = 256;
// The number of palette slots in the SIRC PPU
const int MAX_PALETTE_SIZE = 256;

enum class PaletteReductionBpp { None, FourBpp, TwoBpp };

/**
 * @brief Represents an image in the format supported by the SIRC PPU
 *
 * The SIRC PPU uses a 15 bit (5bpp) color format with a palette.
 * The palette can store 256 colors but usually tile data will
 * only support max 4bpp (16 colors).
 */
class SircImage {

public:
  static SircImage fromQPixmap(const QPixmap &pixmap);
  // TODO: Avoid doing palette reduction in each of the following functions
  [[nodiscard]] QPixmap toQPixmap(const PaletteReductionBpp bpp) const;
  [[nodiscard]] std::vector<QColor>
  getPaletteColors(const PaletteReductionBpp bpp) const;

private:
  std::array<std::array<PaletteReference, HEIGHT_PIXELS>, WIDTH_PIXELS>
      pixelData = {};
  std::vector<SircColor> palette = {};
  std::map<SircColor, size_t> paletteLookup;

  SircImage();
};

#endif // IMAGEPROCESSOR_H
