#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include "inputimage.hpp"

#include <QListWidgetItem>
#include <QMainWindow>
#include <quantizer.hpp>
#include <sircimage.hpp>
#include <vector>

QT_BEGIN_NAMESPACE

namespace Ui {
class MainWindow;
} // namespace Ui
QT_END_NAMESPACE

class MainWindow : public QMainWindow {
  Q_OBJECT

public:
  explicit MainWindow(QWidget *parent = nullptr);
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

  void on_fileList_selectedItemChanged();

private:
  [[nodiscard]] PaletteReductionBpp getPaletteReductionBpp() const;

  // UI Setup
  void setupPaletteReductionOptions() const;
  void setupSourceImageView(const QPixmap &scaledPixmap) const;
  void setupTargetImageView(const SircImage &sircImage) const;
  void setupPaletteView(const SircImage &sircImage) const;
  void loadCurrentImages() const;

  Ui::MainWindow *ui;
  std::vector<QSharedPointer<InputImage>> openedImages;
};
#endif // MAINWINDOW_H
