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

class MainWindow final : public QMainWindow {
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
  void on_actionExportAsm_triggered();

  // Input Image Configuration
  void on_fileList_itemSelectionChanged();
  void on_paletteReductionOptions_currentIndexChanged(int index) const;
  void on_paletteIndexSelection_valueChanged(int value) const;
  void on_moveFileListSelectionUp_clicked() const;
  void on_moveFileListSelectionDown_clicked() const;

private:
  [[nodiscard]] PaletteReductionBpp getPaletteReductionBpp() const;

  // UI Setup
  void setupPaletteReductionOptions() const;
  void setupSourceImageView(const QPixmap &scaledPixmap) const;
  void setupTargetImageView(const SircImage &sircImage) const;
  void setupPaletteView(const SircImage &sircImage) const;
  void loadCurrentImages() const;

  // UI Manipulation
  [[nodiscard]] QList<QListWidgetItem *> sortedSelectedItems() const;
  [[nodiscard]] QList<QListWidgetItem *>
  sortedSelectedItems(Qt::SortOrder sortOrder) const;
  void moveSelectedItems(int offset) const;

  Ui::MainWindow *ui;
  std::unordered_map<InputImageId, std::unique_ptr<InputImage>> openedImages;
  std::vector<InputImageId> selectedImages;
};
#endif // MAINWINDOW_H
