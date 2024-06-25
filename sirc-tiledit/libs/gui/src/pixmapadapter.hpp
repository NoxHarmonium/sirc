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
constexpr unsigned int Q_COLOR_RANGE = 0xFF;
constexpr unsigned int Q_TO_SIRC_COLOR_RATIO = Q_COLOR_RANGE / SIRC_COLOR_RANGE;

class PixmapAdapter {

public:
  static SircImage pixmapToSircImage(const QPixmap &qPixmap);
  static QPixmap sircImageToPixmap(const SircImage &sircImage);
  static std::vector<QColor> getPaletteColors(const SircImage &sircImage);
};

#endif // PIXMAPADAPTER_H
