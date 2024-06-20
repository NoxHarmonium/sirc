#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>

class ImageProcessor;

QT_BEGIN_NAMESPACE
namespace Ui {
class MainWindow;
}
QT_END_NAMESPACE

class MainWindow : public QMainWindow {
  Q_OBJECT

public:
  MainWindow(QWidget *parent = nullptr);
  ~MainWindow() override;

  MainWindow(const MainWindow &) = delete;
  MainWindow &operator=(const MainWindow &) = delete;
  MainWindow(MainWindow &&) noexcept = delete;
  MainWindow &operator=(MainWindow &&) noexcept = delete;

protected:
#ifndef QT_NO_CONTEXTMENU
  void contextMenuEvent(QContextMenuEvent *event) override;
#endif // QT_NO_CONTEXTMENU

private slots:
  // Menu Actions
  void on_actionOpen_triggered();
  void on_actionAbout_triggered();

private:
  // UI Setup
  void setupSourceImageView(const QPixmap &scaledPixmap);
  void setupTargetImageView(const ImageProcessor &imageProcessor);
  void setupPaletteView(const ImageProcessor &imageProcessor);

  Ui::MainWindow *ui;
  QString openedSourceFilename;
};
#endif // MAINWINDOW_H
