#include <QtWidgets>

#include "./ui_mainwindow.h"
#include "aboutdialog.h"
#include "imageprocessor.h"
#include <mainwindow.h>

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent), ui(new Ui::MainWindow) {
  ui->setupUi(this);
}

MainWindow::~MainWindow() { delete ui; }

#ifndef QT_NO_CONTEXTMENU
void MainWindow::contextMenuEvent(QContextMenuEvent *event) {
  QMenu menu(this);
  menu.exec(event->globalPos());
}
#endif // QT_NO_CONTEXTMENU

void MainWindow::on_actionOpen_triggered() {
  openedSourceFilename = QFileDialog::getOpenFileName(
      this, tr("Open source file"), "/home",
      tr("Images (*.png *.xpm *.jpg *.gif *.tif)"));
  auto reader = QImageReader(openedSourceFilename);
  auto pixmap = QPixmap::fromImageReader(&reader);

  auto scaledPixmap =
      pixmap.scaled(WIDTH_PIXELS, HEIGHT_PIXELS, Qt::KeepAspectRatioByExpanding,
                    Qt::FastTransformation);

  // TODO: clang-tidy cppcoreguidelines-owning-memory false positive?
  // NOLINTNEXTLINE
  auto sourceScene = new QGraphicsScene();
  sourceScene->addPixmap(scaledPixmap);
  ui->sourceImageGraphicsView->setScene(sourceScene);

  auto imageProcessor = ImageProcessor::fromQPixmap(&scaledPixmap);
  auto targetPixmap = imageProcessor.toQPixmap();
  auto targetScene = new QGraphicsScene();
  targetScene->addPixmap(targetPixmap);
  ui->targetImageGraphicsView->setScene(targetScene);

  // TODO: Why can't I set this alignment in the UI?
  ui->paletteScrollLayout->setAlignment(Qt::AlignTop);
  auto paletteColors = imageProcessor.getPaletteColors();
  for (size_t i = 0; i < paletteColors.size(); i++) {
    auto color = paletteColors[i];

    auto hWidget = new QWidget();
    hWidget->setMaximumHeight(40);

    auto hLayout = new QHBoxLayout();
    hWidget->setLayout(hLayout);

    auto label = new QLabel(QString("%1: ").arg(i));
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

void MainWindow::on_actionAbout_triggered() {
  // TODO: clang-tidy cppcoreguidelines-owning-memory false positive?
  // NOLINTNEXTLINE
  auto aboutDialog = new AboutDialog(this);
  aboutDialog->setModal(true);
  aboutDialog->show();
}
