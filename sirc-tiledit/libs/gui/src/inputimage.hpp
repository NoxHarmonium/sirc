#ifndef INPUTIMAGE_HPP
#define INPUTIMAGE_HPP

#include "quantizer.hpp"

#include <QString>
#include <qfileinfo.h>

using InputImageId = size_t;

class InputImage {
public:
  InputImage(const QFileInfo &file_info,
             const PaletteReductionBpp output_palette_reduction)
      : fileInfo(file_info), outputPaletteReduction(output_palette_reduction) {}

private:
  QFileInfo fileInfo;
  PaletteReductionBpp outputPaletteReduction = PaletteReductionBpp::None;
  int paletteIndex = 0;

public:
  [[nodiscard]] static InputImageId generateHash(const QFileInfo &fileInfo) {
    // An absolute file path should be unique in a given file system
    return std::hash<QString>{}(fileInfo.absoluteFilePath());
  }

  [[nodiscard]] InputImageId id() const {
    // An absolute file path should be unique in a given file system
    return InputImage::generateHash(fileInfo);
  }
  [[nodiscard]] QFileInfo getFileInfo() const { return fileInfo; }
  [[nodiscard]] PaletteReductionBpp getOutputPaletteReduction() const {
    return outputPaletteReduction;
  }
  [[nodiscard]] int getPaletteIndex() const { return paletteIndex; }

  void setOutputPaletteReduction(const PaletteReductionBpp value) {
    outputPaletteReduction = value;
  }
  void setPaletteIndex(const int value) { paletteIndex = value; }

  friend bool operator==(const InputImage &lhs, const InputImage &rhs) {
    return lhs.fileInfo == rhs.fileInfo;
  }
  friend bool operator!=(const InputImage &lhs, const InputImage &rhs) {
    return !(lhs == rhs);
  }
};

#endif // INPUTIMAGE_HPP
