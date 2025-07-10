#ifndef IMAGEMERGER_HPP
#define IMAGEMERGER_HPP

#include "sircimage.hpp"

class ImageMerger {

public:
  ImageMerger() = default;

  [[nodiscard]] static SircImage
  merge(const std::vector<SircImage> &inputImages);
};

#endif // IMAGEMERGER_HPP
