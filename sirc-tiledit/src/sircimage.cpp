#include "sircimage.h"
#include <QtLogging>
#include <cassert>

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

SircImage::SircImage() = default;

SircImage SircImage::fromQPixmap(const QPixmap &qPixmap) {
  auto sircImage = SircImage();
  auto image = qPixmap.toImage();

  assert(image.width() >= WIDTH_PIXELS && image.height() >= HEIGHT_PIXELS);

  for (int x = 0; x < WIDTH_PIXELS; x++) {
    for (int y = 0; y < HEIGHT_PIXELS; y++) {
      auto pixel = image.pixelColor(x, y);
      auto paletteColor = sircColorFromQRgb(pixel);

      if (auto existingPaletteIndex =
              sircImage.paletteLookup.find(paletteColor);
          existingPaletteIndex != sircImage.paletteLookup.end()) {
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        sircImage.imageData.pixelData[x][y] = existingPaletteIndex->second;
      } else {
        sircImage.imageData.palette.push_back(paletteColor);
        auto paletteIndex = sircImage.imageData.palette.size() - 1;
        sircImage.paletteLookup.insert({paletteColor, paletteIndex});
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
        sircImage.imageData.pixelData[x][y] = paletteIndex;
      }
    }
  }

  qInfo("Total palette entries: %zu", sircImage.imageData.palette.size());

  return sircImage;
}

SircImage SircImage::fromSircImageData(const SircImageData &imageData) {
  auto sircImage = SircImage();
  // TODO: Check if this is a copy
  sircImage.imageData = imageData;

  int i = 0;
  for (auto paletteColor : sircImage.imageData.palette) {
    sircImage.paletteLookup.insert({paletteColor, i++});
  }

  qInfo("Total palette entries: %zu", sircImage.imageData.palette.size());

  return sircImage;
}

QPixmap SircImage::toQPixmap() const {
  auto image = QImage(WIDTH_PIXELS, HEIGHT_PIXELS, QImage::Format_RGB32);

  for (int x = 0; x < WIDTH_PIXELS; x++) {
    for (int y = 0; y < HEIGHT_PIXELS; y++) {
      // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
      auto paletteColor = this->imageData.pixelData[x][y];
      auto sircColor = this->imageData.palette[paletteColor];

      image.setPixelColor(x, y, qRgbFromSircColor(sircColor));
    }
  }
  return QPixmap::fromImage(image);
}

// TODO: Make it more obvious this is QColor colors
std::vector<QColor> SircImage::getPaletteColors() const {
  auto convertedPalette = std::vector<QColor>();

  std::vector<QColor> output;
  std::transform(this->imageData.palette.begin(), this->imageData.palette.end(),
                 std::back_inserter(output),
                 [](SircColor c) { return qRgbFromSircColor(c); });
  return output;
}

// TODO: This breaks encapsulation I suppose, possibly making this class kind of
// pointless. Might need to revisit
SircImageData SircImage::getImageData() const { return this->imageData; }
