#ifndef IMAGEEXPORTER_HPP
#define IMAGEEXPORTER_HPP

#include "sircimage.hpp"
#include <string>
#include <unordered_map>

class ImageExporter {

public:
  ImageExporter() = default;

  [[nodiscard]] static std::string
  exportToAsm(const std::unordered_map<SircPalette, std::vector<SircImage>>
                  &quantizedImagesByPalette,
              uint bpp);
};

#endif // IMAGEEXPORTER_HPP
