
#ifndef MISCADAPTER_HPP
#define MISCADAPTER_HPP

#include "sircimage.hpp"

class MiscAdapter {
public:
  static SircImage
  packedSircPixelDataToSircImage(const PackedSircPixelData &pixelData);
};

#endif // MISCADAPTER_HPP
