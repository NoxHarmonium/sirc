#include <QtWidgets>

#include "./ui_mainwindow.h"
#include "aboutdialog.hpp"
#include "mainwindow.hpp"

#include "inputimage.hpp"
#include "pixmapadapter.hpp"
#include <mediancutquantizer.hpp>

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

void MainWindow::loadCurrentImage() const {
  if (openedImage.isNull()) {
    return;
  }

  const auto openedSourceFilename = openedImage->file_info().filePath();

  qWarning("Opening file: %s", openedSourceFilename.toStdString().c_str());
  auto reader = QImageReader(openedSourceFilename);
  const auto pixmap = QPixmap::fromImageReader(&reader);

  const auto scaledPixmap =
      pixmap.scaled(WIDTH_PIXELS, HEIGHT_PIXELS, Qt::KeepAspectRatioByExpanding,
                    Qt::FastTransformation);

  setupSourceImageView(scaledPixmap);
  const auto sircImage = PixmapAdapter::pixmapToSircImage(scaledPixmap);

  // TODO: palette reduction BPP per item (associate struct with list item??)
  const auto paletteReductionBpp = getPaletteReductionBpp();
  const auto quantizer = MedianCutQuantizer();
  const auto quantizedImage =
      quantizer.quantize(sircImage, paletteReductionBpp);

  setupTargetImageView(quantizedImage);
  setupPaletteView(quantizedImage);
}

// Menu Actions

void MainWindow::on_actionOpen_triggered() {
  auto openedSourceFilename = QFileDialog::getOpenFileName(
      this, tr("Open source file"), "/home",
      tr("Images (*.png *.xpm *.jpg *.gif *.tif)"));

  const auto fileInfo = QFileInfo(openedSourceFilename);
  const auto inputImage = QSharedPointer<InputImage>(
      new InputImage(fileInfo, PaletteReductionBpp::None)
      );
  auto *item = new QListWidgetItem(fileInfo.fileName());
  item->setData(Qt::UserRole, QVariant::fromValue(inputImage));
  ui->fileList->addItem(item);
}

void MainWindow::on_actionAbout_triggered() {
  auto *aboutDialog = new AboutDialog(this);
  aboutDialog->setModal(true);
  aboutDialog->show();
}

void MainWindow::on_fileList_itemSelectionChanged(
    QListWidgetItem *current, [[maybe_unused]] QListWidgetItem *previous) {
  if (current == nullptr) {
    return;
  }
  openedImage = current->data(Qt::UserRole).value<QSharedPointer<InputImage>>();
  loadCurrentImage();
}