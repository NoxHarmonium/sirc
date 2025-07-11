#ifndef INPUTIMAGE_HPP
#define INPUTIMAGE_HPP

#include "quantizer.hpp"

#include <QString>
#include <qfileinfo.h>

class InputImage {
public:
  InputImage(const QFileInfo &file_info,
             PaletteReductionBpp output_palette_reduction)
      : fileInfo(file_info), outputPaletteReduction(output_palette_reduction) {}

private:
  QFileInfo fileInfo;
  PaletteReductionBpp outputPaletteReduction = PaletteReductionBpp::None;

public:
  [[nodiscard]] QFileInfo file_info() const { return fileInfo; }
  [[nodiscard]] PaletteReductionBpp output_palette_reduction() const {
    return outputPaletteReduction;
  }
  void
  set_output_palette_reduction(PaletteReductionBpp output_palette_reduction) {
    outputPaletteReduction = output_palette_reduction;
  }
};

Q_DECLARE_METATYPE(QSharedPointer<InputImage>)

#endif // INPUTIMAGE_HPP
