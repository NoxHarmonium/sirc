#include <QtWidgets>

#include <algorithm>
#include <iostream>
#include <ranges>

#include "./ui_mainwindow.h"

#include "aboutdialog.hpp"
#include "imagemerger.hpp"
#include "inputimage.hpp"
#include "mainwindow.hpp"
#include "pixmapadapter.hpp"

#include <imageexporter.hpp>
#include <mediancutquantizer.hpp>
#include <utils.hpp>

constexpr int PALLETE_VIEW_ITEM_HEIGHT = 40;

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent), ui(new Ui::MainWindow) {
  ui->setupUi(this);
  setupPaletteReductionOptions();
}

MainWindow::~MainWindow() { delete ui; }

#ifndef QT_NO_CONTEXTMENU
void MainWindow::contextMenuEvent(QContextMenuEvent *event) {
  QMenu menu(this);
  menu.exec(event->globalPos());
}
#endif // QT_NO_CONTEXTMENU

PaletteReductionBpp MainWindow::getPaletteSize() const {
  const auto currentItem = ui->paletteReductionOptions->currentData();
  if (currentItem.isNull() || !currentItem.isValid()) {
    return PaletteReductionBpp::None;
  }
  return currentItem.value<PaletteReductionBpp>();
}

std::unordered_map<InputImageId, SircImage>
MainWindow::getOpenedImagesQuantizedById() const {
  std::unordered_map<InputImageId, SircImage> quantizedImagesById;

  // Step 1: Group up images by palettes
  std::unordered_map<size_t, std::vector<InputImage>> paletteGroups;
  for (const auto &openedImage : openedImages | std::views::values) {
    auto &imagesInPaletteGroup = paletteGroups[openedImage->getPaletteIndex()];
    // Future work: Can we avoid this copy?
    imagesInPaletteGroup.push_back(*openedImage);
  }

  // Step 2: Quantize images that share a palette
  for (const auto &paletteGroup : paletteGroups | std::views::values) {
    std::vector<SircImage> imagesToQuantize;
    // TODO: Validate that the palette reduction is the same for the whole
    // group? Does it make sense for different members of the group to have a
    // different reduction value?
    const auto paletteReduction =
        paletteGroup.at(0).getOutputPaletteReduction();
    for (const auto &selectedImage : paletteGroup) {
      const auto openedSourceFilename = selectedImage.getFileInfo().filePath();
      auto reader = QImageReader(openedSourceFilename);
      const auto pixmap = QPixmap::fromImageReader(&reader);

      const auto scaledPixmap =
          pixmap.scaled(WIDTH_PIXELS, HEIGHT_PIXELS,
                        Qt::KeepAspectRatioByExpanding, Qt::FastTransformation);

      const auto sircImage = PixmapAdapter::pixmapToSircImage(scaledPixmap);
      imagesToQuantize.push_back(sircImage);
    }

    const auto quantizer = MedianCutQuantizer();
    const auto quantizedImages =
        quantizer.quantizeAll(imagesToQuantize, paletteReduction);

    for (auto const [index, quantizedImage] : enumerate(quantizedImages)) {
      const auto selectedImage = paletteGroup.at(index);
      quantizedImagesById[selectedImage.id()] = quantizedImage;
    }
  }
  return quantizedImagesById;
}

// UI Setup

void MainWindow::setupPaletteReductionOptions() const {
  ui->paletteReductionOptions->addItem(
      "1:1", QVariant::fromValue(PaletteReductionBpp::None));
  ui->paletteReductionOptions->addItem(
      "4bpp", QVariant::fromValue(PaletteReductionBpp::FourBpp));
  ui->paletteReductionOptions->addItem(
      "2bpp", QVariant::fromValue(PaletteReductionBpp::TwoBpp));
  ui->paletteReductionOptions->setCurrentIndex(0);
}

void MainWindow::setupSourceImageView(const QPixmap &scaledPixmap) const {
  auto *sourceScene = new QGraphicsScene();
  sourceScene->addPixmap(scaledPixmap);
  ui->sourceImageGraphicsView->setScene(sourceScene);
}
void MainWindow::setupTargetImageView(const SircImage &sircImage) const {
  const auto targetPixmap = PixmapAdapter::sircImageToPixmap(sircImage);
  auto *targetScene = new QGraphicsScene();
  targetScene->addPixmap(targetPixmap);
  ui->targetImageGraphicsView->setScene(targetScene);
}

void MainWindow::setupPaletteView(const SircImage &sircImage) const {
  // TODO: Why can't I set this alignment in the UI?
  ui->paletteScrollLayout->setAlignment(Qt::AlignTop);
  for (auto children = ui->paletteScrollContents->findChildren<QWidget *>();
       auto *child : children) {
    child->deleteLater();
  }

  int paletteIndex = 0;
  for (auto color : PixmapAdapter::getPaletteColors(sircImage)) {
    auto *hWidget = new QWidget();
    hWidget->setMaximumHeight(PALLETE_VIEW_ITEM_HEIGHT);

    auto *hLayout = new QHBoxLayout();
    hWidget->setLayout(hLayout);

    auto *label = new QLabel(QString("%1: ").arg(paletteIndex++));
    auto *colorIndicator = new QFrame();

    auto pal = QPalette();
    pal.setColor(QPalette::Window, color);
    colorIndicator->setAutoFillBackground(true);
    colorIndicator->setPalette(pal);

    hLayout->addWidget(label);
    hLayout->addWidget(colorIndicator);
    ui->paletteScrollLayout->addWidget(hWidget);
  }
}

void MainWindow::loadCurrentImages() const {
  auto const quantizedImagesById = this->getOpenedImagesQuantizedById();

  std::vector<SircImage> selectedQuantizedImages;
  selectedQuantizedImages.reserve(selectedImages.size());
  for (const auto &selectedImage : selectedImages) {
    selectedQuantizedImages.push_back(quantizedImagesById.at(selectedImage));
  }
  const auto mergedImage = ImageMerger::merge(selectedQuantizedImages);

  setupTargetImageView(mergedImage);
  setupPaletteView(mergedImage);
}

// UI Manipulation
void MainWindow::moveSelectedItems(const int offset) const {

  auto selectedItems =
      sortedSelectedItems(offset >= 0 ? Qt::SortOrder::AscendingOrder
                                      : Qt::SortOrder::DescendingOrder);
  for (QListWidgetItem *selectedItem : selectedItems) {
    if (selectedItem == nullptr) {
      continue;
    }
    const auto currentIndex = ui->fileList->indexFromItem(selectedItem);
    const auto row = currentIndex.row();
    if (const auto newRow = row + offset;
        newRow < 0 || newRow >= ui->fileList->count()) {
      return;
    }
    auto *const taken = ui->fileList->takeItem(row);
    if (taken) {
      ui->fileList->insertItem(row + offset, selectedItem);
      selectedItem->setSelected(true);
    }
  }
}

QList<QListWidgetItem *> MainWindow::sortedSelectedItems() const {
  return sortedSelectedItems(Qt::SortOrder::AscendingOrder);
}

QList<QListWidgetItem *>
MainWindow::sortedSelectedItems(Qt::SortOrder sortOrder) const {
  auto selectedItems = ui->fileList->selectedItems();
  std::ranges::sort(selectedItems, [this, sortOrder](const QListWidgetItem *a,
                                                     const QListWidgetItem *b) {
    const int rowA = ui->fileList->row(a);
    const int rowB = ui->fileList->row(b);
    return sortOrder == Qt::SortOrder::AscendingOrder ? rowA > rowB
                                                      : rowA < rowB;
  });
  return selectedItems;
}

// Menu Actions

void MainWindow::on_actionOpen_triggered() {
  auto openedSourceFilenames = QFileDialog::getOpenFileNames(
      this, tr("Open source file"), "/home",
      tr("Images (*.png *.xpm *.jpg *.gif *.tif)"));

  for (const auto &openedSourceFilename : openedSourceFilenames) {
    const auto fileInfo = QFileInfo(openedSourceFilename);
    const auto hash = InputImage::generateHash(fileInfo);
    auto *item = new QListWidgetItem(fileInfo.fileName());
    item->setData(Qt::UserRole, QVariant::fromValue(hash));
    openedImages.emplace(hash, std::make_unique<InputImage>(
                                   fileInfo, PaletteReductionBpp::None));

    ui->fileList->addItem(item);
  }
}

void MainWindow::on_actionAbout_triggered() {
  auto *aboutDialog = new AboutDialog(this);
  aboutDialog->setModal(true);
  aboutDialog->show();
}

void MainWindow::on_actionExportAsm_triggered() {
  const auto quantizedImagesById = this->getOpenedImagesQuantizedById();
  std::unordered_map<SircPalette,
                     std::vector<std::pair<std::string, SircImage>>>
      quantizedImagesByPalette;
  for (const auto &[id, quantizedImage] : quantizedImagesById) {
    const auto &image = this->openedImages.at(id);
    const std::string name = image->getFileInfo().fileName().toStdString();
    quantizedImagesByPalette[quantizedImage.palette].emplace_back(
        name, quantizedImage);
  }

  const auto asmOutputStr =
      ImageExporter::exportToAsm(quantizedImagesByPalette);

  auto filenameToSave = QFileDialog::getSaveFileName(
      this, tr("Save file"), "/home", tr("Assembly File (*.asm)"));

  QFile file(filenameToSave);
  // TODO: Error handling
  if (file.open(QIODevice::WriteOnly | QIODevice::Truncate)) {
    QTextStream stream(&file);
    stream << QString(
                  ";; Warning: Exported by sirc-tiledit. Don't edit manually.")
           << Qt::endl
           << QString::fromStdString(asmOutputStr) << Qt::endl;
  }
  file.close();
}

// Input Image Configuration

void MainWindow::on_fileList_itemSelectionChanged() {
  selectedImages.clear();
  auto selectedItems = sortedSelectedItems();
  for (const auto &selectedItem : selectedItems) {
    if (selectedItem == nullptr) {
      continue;
    }
    selectedImages.push_back(
        selectedItem->data(Qt::UserRole).value<InputImageId>());
  }
  loadCurrentImages();
}

void MainWindow::on_paletteReductionOptions_currentIndexChanged(
    [[maybe_unused]] int index) const {
  const auto selectedBpp = getPaletteSize();

  for (const auto &openedImageId : selectedImages) {
    openedImages.at(openedImageId)->setOutputPaletteReduction(selectedBpp);
  }
  loadCurrentImages();
}

void MainWindow::on_paletteIndexSelection_valueChanged(int value) const {
  for (const auto &openedImageId : selectedImages) {
    openedImages.at(openedImageId)->setPaletteIndex(value);
  }
  loadCurrentImages();
}

void MainWindow::on_moveFileListSelectionUp_clicked() const {
  moveSelectedItems(-1);
}

void MainWindow::on_moveFileListSelectionDown_clicked() const {
  moveSelectedItems(1);
}
