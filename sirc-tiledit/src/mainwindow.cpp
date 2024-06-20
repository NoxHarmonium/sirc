#include <QtWidgets>

#include "./ui_mainwindow.h"
#include "aboutdialog.h"
#include <mainwindow.h>

const int SOURCE_IMAGE_PADDING = 10;

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
      pixmap.scaled(ui->sourceImageGraphicsView->size().shrunkBy(
                        QMargins(SOURCE_IMAGE_PADDING, SOURCE_IMAGE_PADDING,
                                 SOURCE_IMAGE_PADDING, SOURCE_IMAGE_PADDING)),
                    Qt::KeepAspectRatio, Qt::FastTransformation);

  // TODO: clang-tidy cppcoreguidelines-owning-memory false positive?
  // NOLINTNEXTLINE
  auto scene = new QGraphicsScene();
  scene->addPixmap(scaledPixmap);
  ui->sourceImageGraphicsView->setScene(scene);
}

void MainWindow::on_actionAbout_triggered() {
  // TODO: clang-tidy cppcoreguidelines-owning-memory false positive?
  // NOLINTNEXTLINE
  auto aboutDialog = new AboutDialog(this);
  aboutDialog->setModal(true);
  aboutDialog->show();
}
