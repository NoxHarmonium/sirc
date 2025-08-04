#include <QtWidgets>
#include <libsirc/libsirc.h>

#include "./ui_mainwindow.h"
#include "aboutdialog.hpp"
#include "mainwindow.hpp"

#include "imagemerger.hpp"
#include "inputimage.hpp"
#include "pixmapadapter.hpp"

#include <algorithm>
#include <iostream>
#include <mediancutquantizer.hpp>
#include <ranges>

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

PaletteReductionBpp MainWindow::getPaletteReductionBpp() const {
  const auto currentItem = ui->paletteReductionOptions->currentData();
  if (currentItem.isNull() || !currentItem.isValid()) {
    return PaletteReductionBpp::None;
  }
  return currentItem.value<PaletteReductionBpp>();
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
  auto quantizedImages = std::vector<SircImage>();
  for (const auto &openedImage : openedImages) {
    if (openedImage.isNull()) {
      return;
    }

    const auto openedSourceFilename = openedImage->file_info().filePath();

    auto reader = QImageReader(openedSourceFilename);
    const auto pixmap = QPixmap::fromImageReader(&reader);

    const auto scaledPixmap =
        pixmap.scaled(WIDTH_PIXELS, HEIGHT_PIXELS,
                      Qt::KeepAspectRatioByExpanding, Qt::FastTransformation);

    setupSourceImageView(scaledPixmap);
    const auto sircImage = PixmapAdapter::pixmapToSircImage(scaledPixmap);

    const auto paletteReductionBpp = openedImage->output_palette_reduction();
    const auto quantizer = MedianCutQuantizer();
    const auto quantizedImage =
        quantizer.quantize(sircImage, paletteReductionBpp);

    quantizedImages.push_back(quantizedImage);
  }

  const auto mergedImage = ImageMerger::merge(quantizedImages);

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
    const auto inputImage = QSharedPointer<InputImage>(
        new InputImage(fileInfo, PaletteReductionBpp::None));
    auto *item = new QListWidgetItem(fileInfo.fileName());
    item->setData(Qt::UserRole, QVariant::fromValue(inputImage));
    ui->fileList->addItem(item);
  }
}

void MainWindow::on_actionAbout_triggered() {
  auto *aboutDialog = new AboutDialog(this);
  aboutDialog->setModal(true);
  aboutDialog->show();
}

void MainWindow::on_actionExportAsm_triggered() {
  constexpr std::array<uint16_t, 4> pixel_data = {0, 1, 2, 3};

  const auto tilemap =
      libsirc::CTilemap{.label = "some_label",
                        .comment = "some_comment",
                        .palette_index = 0,
                        .packed_pixel_data = pixel_data.data(),
                        .packed_pixel_data_len = pixel_data.size()};

  const std::array tilemaps = {tilemap};

  const libsirc::CPalette palette = {
      .comment = "palette comment",
      .data = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15}};

  auto export_data = libsirc::CTilemapExport{
      .tilemaps = tilemaps.data(),
      .tilemaps_len = tilemaps.size(),
      .palette_label = "some_label",
      .palettes = {palette, palette, palette, palette, palette, palette,
                   palette, palette, palette, palette, palette, palette,
                   palette, palette, palette, palette}};

  char *asmChar = libsirc::tilemap_to_str(libsirc::CTilemapExport(export_data));
  const std::string asmOutputStr(asmChar);
  std::cout << asmOutputStr << '\n';
  libsirc::free_str(asmChar);
}

// Input Image Configuration

void MainWindow::on_fileList_itemSelectionChanged() {
  openedImages = std::vector<QSharedPointer<InputImage>>();
  auto selectedItems = sortedSelectedItems();
  for (const auto &selectedItem : selectedItems) {
    if (selectedItem == nullptr) {
      continue;
    }
    openedImages.push_back(
        selectedItem->data(Qt::UserRole).value<QSharedPointer<InputImage>>());
  }
  loadCurrentImages();
}

void MainWindow::on_paletteReductionOptions_currentIndexChanged(
    [[maybe_unused]] int index) const {
  const auto selectedBpp = getPaletteReductionBpp();

  for (const auto &openedImage : openedImages) {
    if (openedImage == nullptr) {
      continue;
    }
    openedImage->set_output_palette_reduction(selectedBpp);
  }
  loadCurrentImages();
}

void MainWindow::on_moveFileListSelectionUp_clicked() const {
  moveSelectedItems(-1);
}

void MainWindow::on_moveFileListSelectionDown_clicked() const {
  moveSelectedItems(1);
}
