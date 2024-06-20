#include "sircimage.h"
#include <QtLogging>
#include <cassert>

// QColor uses standard 32 bit colour ARGB (8bpp)
const unsigned int Q_COLOR_RANGE = 0xFF;
// SIRC uses a packed 16 bit color RGB (5bpp)
const unsigned int SIRC_COLOR_COMPONENT_BITS = 5;
const unsigned int SIRC_COLOR_RANGE = (1 << (SIRC_COLOR_COMPONENT_BITS)) - 1;
const unsigned int Q_TO_SIRC_COLOR_RATIO = Q_COLOR_RANGE / SIRC_COLOR_RANGE;

u_int16_t sircColorFromQRgb(const QColor qColor) {
  const unsigned int r = qColor.red() / Q_TO_SIRC_COLOR_RATIO;
  const unsigned int g = qColor.green() / Q_TO_SIRC_COLOR_RATIO;
  const unsigned int b = qColor.blue() / Q_TO_SIRC_COLOR_RATIO;

  return r << SIRC_COLOR_COMPONENT_BITS * 2 | g << SIRC_COLOR_COMPONENT_BITS |
         b;
}

QColor qRgbFromSircColor(const u_int16_t sircColor) {
  const unsigned int sircR =
      (sircColor >> (SIRC_COLOR_COMPONENT_BITS * 2)) & SIRC_COLOR_RANGE;
  const unsigned int sircG =
      (sircColor >> SIRC_COLOR_COMPONENT_BITS) & SIRC_COLOR_RANGE;
  const unsigned int sircB = sircColor & SIRC_COLOR_RANGE;

  QColor qColor;

  qColor.setRed((int)(sircR * Q_TO_SIRC_COLOR_RATIO));
  qColor.setGreen((int)(sircG * Q_TO_SIRC_COLOR_RATIO));
  qColor.setBlue((int)(sircB * Q_TO_SIRC_COLOR_RATIO));

  return qColor;
}

QColor qRgbFromSircColorWithReduction(const u_int16_t sircColor,
                                      const PaletteReductionBpp bpp) {
  switch (bpp) {
  case PaletteReductionBpp::None:
    return qRgbFromSircColor(sircColor);
  case PaletteReductionBpp::FourBpp:
  case PaletteReductionBpp::TwoBpp:
    // TODO: PaletteReduction
    return {};
  }
}

SircImage::SircImage() = default;

SircImage SircImage::fromQPixmap(const QPixmap &qPixmap) {
  auto imageProcessor = SircImage();
  auto image = qPixmap.toImage();

  assert(image.width() >= WIDTH_PIXELS && image.height() >= HEIGHT_PIXELS);

  for (int x = 0; x < WIDTH_PIXELS; x++) {
    for (int y = 0; y < HEIGHT_PIXELS; y++) {
      auto pixel = image.pixelColor(x, y);
      auto paletteColor = sircColorFromQRgb(pixel);

      if (auto existingPaletteIndex =
              imageProcessor.paletteLookup.find(paletteColor);
          existingPaletteIndex != imageProcessor.paletteLookup.end()) {
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        imageProcessor.pixelData[x][y] = existingPaletteIndex->second;
      } else {
        imageProcessor.palette.push_back(paletteColor);
        auto paletteIndex = imageProcessor.palette.size() - 1;
        imageProcessor.paletteLookup.insert({paletteColor, paletteIndex});
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        imageProcessor.pixelData[x][y] = paletteIndex;
      }
    }
  }

  qDebug("Total palette entries: %zu", imageProcessor.palette.size());

  return imageProcessor;
}

QPixmap SircImage::toQPixmap(const PaletteReductionBpp bpp) const {
  auto image = QImage(WIDTH_PIXELS, HEIGHT_PIXELS, QImage::Format_RGB32);

  for (int x = 0; x < WIDTH_PIXELS; x++) {
    for (int y = 0; y < HEIGHT_PIXELS; y++) {
      // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
      auto paletteColor = this->pixelData[x][y];
      auto sircColor = this->palette[paletteColor];

      image.setPixelColor(x, y, qRgbFromSircColorWithReduction(sircColor, bpp));
    }
  }
  return QPixmap::fromImage(image);
}

std::vector<QColor>
SircImage::getPaletteColors(const PaletteReductionBpp bpp) const {
  auto convertedPalette = std::vector<QColor>();

  std::vector<QColor> output;
  std::transform(
      this->palette.begin(), this->palette.end(), std::back_inserter(output),
      [&bpp](SircColor c) { return qRgbFromSircColorWithReduction(c, bpp); });
  return output;
}
