#ifndef IMAGEEXPORTER_HPP
#define IMAGEEXPORTER_HPP

#include "sircimage.hpp"
#include <string>
#include <unordered_map>

class ImageExporter {

public:
  ImageExporter() = default;

  [[nodiscard]] static std::string exportToAsm(
      // TODO: Should the std::pair be given a type alias or something?
      const std::unordered_map<SircPalette,
                               std::vector<std::pair<std::string, SircImage>>>
          &quantizedImagesByPalette);
};

#endif // IMAGEEXPORTER_HPP
