
#ifndef IMAGELOADER_HPP
#define IMAGELOADER_HPP

#include "constants.hpp"

#include <array>
#include <cstdint>
#include <limits>

using RgbaComponent = uint8_t;
using RgbaPixel = uint32_t;
using RgbaPixelData =
    std::array<std::array<RgbaPixel, HEIGHT_PIXELS>, WIDTH_PIXELS>;

constexpr RgbaComponent RGBA_COMPONENT_MIN =
    std::numeric_limits<RgbaComponent>::min();
constexpr RgbaComponent RGBA_COMPONENT_MAX =
    std::numeric_limits<RgbaComponent>::max();

class ImageLoader {
public:
  static RgbaPixelData loadImageFromPng(const char *filename);
  static void saveImageToPng(const char *filename, const RgbaPixelData &data);
};

#endif // IMAGELOADER_HPP
