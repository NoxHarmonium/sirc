#ifndef ABOUTDIALOG_H
#define ABOUTDIALOG_H

#include <QDialog>

namespace Ui {
class AboutDialog;
}

class AboutDialog : public QDialog {
  Q_OBJECT

public:
  explicit AboutDialog(QWidget *parent = nullptr);
  ~AboutDialog() override;

  AboutDialog(const AboutDialog &) = delete;
  AboutDialog &operator=(const AboutDialog &) = delete;
  AboutDialog(AboutDialog &&) noexcept = delete;
  AboutDialog &operator=(AboutDialog &&) noexcept = delete;

private:
  Ui::AboutDialog *ui;
};

#endif // ABOUTDIALOG_H
