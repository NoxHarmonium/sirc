
#include "pixmapadapter.hpp"
#include "sircimage.hpp"

#include <miscadapter.hpp>

SircColor sircColorFromQRgb(const QColor qColor) {
  const unsigned int r = qColor.red() / Q_TO_SIRC_COLOR_RATIO;
  const unsigned int g = qColor.green() / Q_TO_SIRC_COLOR_RATIO;
  const unsigned int b = qColor.blue() / Q_TO_SIRC_COLOR_RATIO;

  return r << SIRC_COLOR_COMPONENT_BITS * 2 | g << SIRC_COLOR_COMPONENT_BITS |
         b;
}

QColor qRgbFromSircColor(const SircColor sircColor) {
  const unsigned int sircR =
      sircColor >> SIRC_COLOR_COMPONENT_BITS * 2 & SIRC_COLOR_RANGE;
  const unsigned int sircG =
      sircColor >> SIRC_COLOR_COMPONENT_BITS & SIRC_COLOR_RANGE;
  const unsigned int sircB = sircColor & SIRC_COLOR_RANGE;

  QColor qColor;

  qColor.setRed(static_cast<int>(sircR * Q_TO_SIRC_COLOR_RATIO));
  qColor.setGreen(static_cast<int>(sircG * Q_TO_SIRC_COLOR_RATIO));
  qColor.setBlue(static_cast<int>(sircB * Q_TO_SIRC_COLOR_RATIO));

  return qColor;
}
// TODO: Export assembly

SircImage PixmapAdapter::pixmapToSircImage(const QPixmap &qPixmap) {
  const auto image = qPixmap.toImage();
  PackedSircPixelData pixelData;
  assert(image.width() >= WIDTH_PIXELS && image.height() >= HEIGHT_PIXELS);

  for (int x = 0; x < WIDTH_PIXELS; x++) {
    for (int y = 0; y < HEIGHT_PIXELS; y++) {
      const auto pixel = image.pixelColor(x, y);
      const auto convertedPixel = sircColorFromQRgb(pixel);
      // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
      pixelData[x][y] = convertedPixel;
    }
  }
  auto sircImage = MiscAdapter::packedSircPixelDataToSircImage(pixelData);

  return sircImage;
}

QPixmap PixmapAdapter::sircImageToPixmap(const SircImage &sircImage) {
  auto image = QImage(WIDTH_PIXELS, HEIGHT_PIXELS, QImage::Format_RGB32);
  auto [palette, pixelData] = sircImage;

  for (int x = 0; x < WIDTH_PIXELS; x++) {
    for (int y = 0; y < HEIGHT_PIXELS; y++) {
      // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-constant-array-index)
      const auto paletteColor = pixelData[(x * WIDTH_PIXELS) + y];
      assert(paletteColor <= palette.size());
      const auto sircColor = palette.size() == 0 ? 0 : palette[paletteColor];

      image.setPixelColor(x, y, qRgbFromSircColor(sircColor));
    }
  }
  return QPixmap::fromImage(image);
}

std::vector<QColor>
PixmapAdapter::getPaletteColors(const SircImage &sircImage) {
  auto convertedPalette = std::vector<QColor>();
  const auto [palette, pixelData] = sircImage;

  std::vector<QColor> output;
  std::ranges::transform(
      palette, std::back_inserter(output),
      [](const SircColor c) { return qRgbFromSircColor(c); });
  return output;
}