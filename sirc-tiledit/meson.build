project('sirc-tiledit', 'cpp', version: '1.0.0', license: 'AGPL-3.0-only')
add_global_arguments(language: 'cpp')

qt6 = import('qt6')
qt6_dep = dependency('qt6', modules: ['Core', 'Gui', 'Widgets'])

gui_project_dependencies = [qt6_dep]
gui_source_files = [
  'libs/gui/src/mainwindow.cpp',
  'libs/gui/src/aboutdialog.cpp',
  'libs/gui/src/main.cpp',
  'libs/gui/src/pixmapadapter.cpp',
]

shared_library_source_files = ['libs/shared/src/sircimage.cpp', 'libs/shared/src/mediancutquantizer.cpp']
shared_library_include_files = include_directories('libs/shared/include')

moc_files = qt6.compile_moc(
  headers: ['libs/gui/src/mainwindow.hpp', 'libs/gui/src/aboutdialog.hpp'],
  dependencies: qt6_dep,
)

compiled_ui_files = qt6.compile_ui(sources: ['libs/gui/ui/mainwindow.ui', 'libs/gui/ui/aboutdialog.ui'])

build_args = [
  '-DPROJECT_NAME=' + meson.project_name(),
  '-DPROJECT_VERSION=' + meson.project_version(),
  '-std=c++20',
  '-Wall',
  '-Werror',
  '-Wshadow',
  '-Wextra',
  '-Wpedantic',
]

shared_lib = library(
  meson.project_name() + '-shared',
  shared_library_source_files,
  version: meson.project_version(),
  include_directories: shared_library_include_files,
  cpp_args: build_args,
)

executable(
  meson.project_name() + '-gui',
  moc_files,
  gui_source_files,
  compiled_ui_files,
  dependencies: gui_project_dependencies,
  include_directories: shared_library_include_files,
  cpp_args: build_args,
  link_with: shared_lib,
)