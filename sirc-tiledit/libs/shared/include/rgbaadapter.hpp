#ifndef RGBAADAPTER_H
#define RGBAADAPTER_H

#include <imageloader.hpp>
#include <sircimage.hpp>

// PNGs are loaded with standard 32 bit colour RGBA (8bpp)
constexpr unsigned int RGBA_COLOR_RANGE = 0xFF;
constexpr unsigned int RGBA_TO_SIRC_COLOR_RATIO =
    RGBA_COLOR_RANGE / SIRC_COLOR_RANGE;
constexpr unsigned int RGBA_BLACK = 0x000000FF;

class RgbaAdapter {
public:
  static SircImage rgbaToSircImage(const RgbaPixelData &pixelData);
  static RgbaPixelData sircImageToRgba(const SircImage &sircImage);
};

#endif // RGBAADAPTER_H
