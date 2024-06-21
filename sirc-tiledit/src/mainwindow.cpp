#include <QtWidgets>

#include "./ui_mainwindow.h"
#include "aboutdialog.h"
#include "mediancutquantizer.h"
#include <mainwindow.h>

const int PALLETE_VIEW_ITEM_HEIGHT = 40;

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
  auto currentItem = ui->paletteReductionOptions->currentData();
  if (currentItem.isNull() || !currentItem.isValid()) {
    return PaletteReductionBpp::None;
  }
  return currentItem.value<PaletteReductionBpp>();
}

// UI Setup

void MainWindow::setupPaletteReductionOptions() {
  ui->paletteReductionOptions->addItem(
      "1:1", QVariant::fromValue(PaletteReductionBpp::None));
  ui->paletteReductionOptions->addItem(
      "4bpp", QVariant::fromValue(PaletteReductionBpp::FourBpp));
  ui->paletteReductionOptions->addItem(
      "2bpp", QVariant::fromValue(PaletteReductionBpp::TwoBpp));
  ui->paletteReductionOptions->setCurrentIndex(0);
}

void MainWindow::setupSourceImageView(const QPixmap &scaledPixmap) {
  auto sourceScene = new QGraphicsScene();
  sourceScene->addPixmap(scaledPixmap);
  ui->sourceImageGraphicsView->setScene(sourceScene);
}
void MainWindow::setupTargetImageView(const SircImage &imageProcessor) {
  auto targetPixmap = imageProcessor.toQPixmap();
  auto targetScene = new QGraphicsScene();
  targetScene->addPixmap(targetPixmap);
  ui->targetImageGraphicsView->setScene(targetScene);
}

void MainWindow::setupPaletteView(const SircImage &imageProcessor) {
  // TODO: Why can't I set this alignment in the UI?
  ui->paletteScrollLayout->setAlignment(Qt::AlignTop);
  auto children = ui->paletteScrollContents->findChildren<QWidget *>();
  for (auto child : children) {
    child->deleteLater();
  }

  int paletteIndex = 0;
  for (auto color : imageProcessor.getPaletteColors()) {
    auto hWidget = new QWidget();
    hWidget->setMaximumHeight(PALLETE_VIEW_ITEM_HEIGHT);

    auto hLayout = new QHBoxLayout();
    hWidget->setLayout(hLayout);

    auto label = new QLabel(QString("%1: ").arg(paletteIndex++));
    auto colorIndicator = new QFrame();

    QPalette pal = QPalette();
    pal.setColor(QPalette::Window, color);
    colorIndicator->setAutoFillBackground(true);
    colorIndicator->setPalette(pal);

    hLayout->addWidget(label);
    hLayout->addWidget(colorIndicator);
    ui->paletteScrollLayout->addWidget(hWidget);
  }
}

// Menu Actions

void MainWindow::on_actionOpen_triggered() {
  openedSourceFilename = QFileDialog::getOpenFileName(
      this, tr("Open source file"), "/home",
      tr("Images (*.png *.xpm *.jpg *.gif *.tif)"));
  auto reader = QImageReader(openedSourceFilename);
  auto pixmap = QPixmap::fromImageReader(&reader);

  auto scaledPixmap =
      pixmap.scaled(WIDTH_PIXELS, HEIGHT_PIXELS, Qt::KeepAspectRatioByExpanding,
                    Qt::FastTransformation);

  setupSourceImageView(scaledPixmap);

  auto sircImage = SircImage::fromQPixmap(scaledPixmap);

  qWarning("Opening file: %s", openedSourceFilename.toStdString().c_str());

  auto paletteReductionBpp = getPaletteReductionBpp();
  auto quantizer = MedianCutQuantizer();
  auto quantizedImage = quantizer.quantize(sircImage, paletteReductionBpp);

  setupTargetImageView(quantizedImage);
  setupPaletteView(quantizedImage);
}

void MainWindow::on_actionAbout_triggered() {
  auto aboutDialog = new AboutDialog(this);
  aboutDialog->setModal(true);
  aboutDialog->show();
}
