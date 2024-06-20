#ifndef IMAGEPROCESSOR_H
#define IMAGEPROCESSOR_H

#include <QImage>
#include <QPixmap>
#include <map>

const int WIDTH_PIXELS = 256;
const int HEIGHT_PIXELS = 256;
// The number of palette slots in the SIRC PPU
const int MAX_PALETTE_SIZE = 256;

class ImageProcessor {

public:
  static ImageProcessor fromQPixmap(QPixmap *const qPixmap);
  QPixmap toQPixmap() const;

private:
  u_int8_t pixelData[WIDTH_PIXELS][HEIGHT_PIXELS] = {};
  u_int16_t palette[MAX_PALETTE_SIZE] = {};
  std::map<u_int16_t, size_t> paletteLookup;

  ImageProcessor();
};

#endif // IMAGEPROCESSOR_H
