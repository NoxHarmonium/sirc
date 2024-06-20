#include "imageprocessor.h"
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

QColor qRgbFromSircColor(u_int16_t color) {
  const unsigned int sircR =
      (color >> (SIRC_COLOR_COMPONENT_BITS * 2)) & SIRC_COLOR_RANGE;
  const unsigned int sircG =
      (color >> SIRC_COLOR_COMPONENT_BITS) & SIRC_COLOR_RANGE;
  const unsigned int sircB = color & SIRC_COLOR_RANGE;

  QColor qColor;

  qColor.setRed((int)(sircR * Q_TO_SIRC_COLOR_RATIO));
  qColor.setGreen((int)(sircG * Q_TO_SIRC_COLOR_RATIO));
  qColor.setBlue((int)(sircB * Q_TO_SIRC_COLOR_RATIO));

  return qColor;
}

ImageProcessor::ImageProcessor() = default;

ImageProcessor ImageProcessor::fromQPixmap(QPixmap *const qPixmap) {
  auto imageProcessor = ImageProcessor();
  auto image = qPixmap->toImage();

  assert(image.width() >= WIDTH_PIXELS && image.height() >= HEIGHT_PIXELS);

  for (int x = 0; x < WIDTH_PIXELS; x++) {
    for (int y = 0; y < HEIGHT_PIXELS; y++) {
      auto pixel = image.pixelColor(x, y);
      auto paletteColor = sircColorFromQRgb(pixel);

      if (auto existingPaletteIndex =
              imageProcessor.paletteLookup.find(paletteColor);
          existingPaletteIndex != imageProcessor.paletteLookup.end()) {
        imageProcessor.pixelData[x][y] = existingPaletteIndex->second;
      } else {
        imageProcessor.palette.push_back(paletteColor);
        auto paletteIndex = imageProcessor.palette.size() - 1;
        imageProcessor.paletteLookup.insert({paletteColor, paletteIndex});
        imageProcessor.pixelData[x][y] = paletteIndex;
      }
    }
  }

  qDebug("Total palette entries: %zu", imageProcessor.palette.size());

  return imageProcessor;
}

QPixmap ImageProcessor::toQPixmap() const {
  auto image = QImage(WIDTH_PIXELS, HEIGHT_PIXELS, QImage::Format_RGB32);

  for (int x = 0; x < WIDTH_PIXELS; x++) {
    for (int y = 0; y < HEIGHT_PIXELS; y++) {
      auto paletteColor = this->pixelData[x][y];
      auto sircColor = this->palette[paletteColor];

      image.setPixelColor(x, y, qRgbFromSircColor(sircColor));
    }
  }
  return QPixmap::fromImage(image);
}

std::vector<QColor> ImageProcessor::getPaletteColors() const {
  auto convertedPalette = std::vector<QColor>();

  std::vector<QColor> output;
  std::transform(this->palette.begin(), this->palette.end(),
                 std::back_inserter(output),
                 [](SircColor c) { return qRgbFromSircColor(c); });
  return output;
}
