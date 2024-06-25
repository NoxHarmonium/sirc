#ifndef PIXMAPADAPTER_H
#define PIXMAPADAPTER_H

#include <QImage>
#include <QPixmap>
#include <array>
#include <cstdint>
#include <map>
#include <vector>

#include <sircimage.hpp>

// QColor uses standard 32 bit colour ARGB (8bpp)
const unsigned int Q_COLOR_RANGE = 0xFF;
const unsigned int Q_TO_SIRC_COLOR_RATIO = Q_COLOR_RANGE / SIRC_COLOR_RANGE;

class PixmapAdapter {

public:
  static SircImage pixmapToSircImage(const QPixmap &pixmap);
  static QPixmap sircImageToPixmap(const SircImage &pixmap);
  static std::vector<QColor> getPaletteColors(const SircImage &pixmap);
};

#endif // PIXMAPADAPTER_H
