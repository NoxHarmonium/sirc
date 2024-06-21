#ifndef IMAGEPROCESSOR_H
#define IMAGEPROCESSOR_H

#include <QImage>
#include <QPixmap>
#include <array>
#include <cstdint>
#include <map>
#include <vector>

const int WIDTH_PIXELS = 256;
const int HEIGHT_PIXELS = 256;
// The number of palette slots in the SIRC PPU
const int MAX_PALETTE_SIZE = 256;

using SircColor = uint16_t;
using PaletteReference = uint8_t;
using PixelData =
    std::array<std::array<PaletteReference, HEIGHT_PIXELS>, WIDTH_PIXELS>;

struct SircImageData {
  std::vector<SircColor> palette;
  PixelData pixelData;
};

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
  static SircImage fromSircImageData(const SircImageData &imageData);

  // TODO: Avoid doing palette reduction in each of the following functions
  [[nodiscard]] QPixmap toQPixmap() const;
  [[nodiscard]] std::vector<QColor> getPaletteColors() const;

private:
  SircImageData imageData = {};
  std::map<SircColor, size_t> paletteLookup;

  SircImage();
};

#endif // IMAGEPROCESSOR_H
