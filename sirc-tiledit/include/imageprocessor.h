#ifndef IMAGEPROCESSOR_H
#define IMAGEPROCESSOR_H

#include <QImage>
#include <QPixmap>
#include <map>
#include <vector>

typedef u_int16_t SircColor;
typedef u_int8_t PaletteReference;

const int WIDTH_PIXELS = 256;
const int HEIGHT_PIXELS = 256;
// The number of palette slots in the SIRC PPU
const int MAX_PALETTE_SIZE = 256;

class ImageProcessor {

public:
  static ImageProcessor fromQPixmap(QPixmap *const qPixmap);
  QPixmap toQPixmap() const;
  std::vector<QColor> getPaletteColors() const;

private:
  PaletteReference pixelData[WIDTH_PIXELS][HEIGHT_PIXELS] = {};
  std::vector<SircColor> palette = {};
  std::map<SircColor, size_t> paletteLookup;

  ImageProcessor();
};

#endif // IMAGEPROCESSOR_H
