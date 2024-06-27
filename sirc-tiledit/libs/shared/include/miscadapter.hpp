
#ifndef MISCADAPTER_HPP
#define MISCADAPTER_HPP

#include "sircimage.hpp"


class MiscAdapter {
public:
  static SircImage fromPixelData(const PackedPixelData &pixelData);
  static SircImage fromSircImageData(const SircImageData &imageData);
};



#endif //MISCADAPTER_HPP
