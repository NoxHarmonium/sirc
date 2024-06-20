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

class ImageProcessor {

public:
  static ImageProcessor fromQPixmap(const QPixmap &pixmap);
  [[nodiscard]] QPixmap toQPixmap() const;
  [[nodiscard]] std::vector<QColor> getPaletteColors() const;

private:
  std::array<std::array<PaletteReference, HEIGHT_PIXELS>, WIDTH_PIXELS>
      pixelData = {};
  std::vector<SircColor> palette = {};
  std::map<SircColor, size_t> paletteLookup;

  ImageProcessor();
};

#endif // IMAGEPROCESSOR_H
