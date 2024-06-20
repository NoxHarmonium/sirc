project(
  'sirc-tiledit', 
  'cpp',
  version : '1.0.0',
  license : 'AGPL-3.0-only'
)
add_global_arguments(language : 'cpp')

qt6 = import('qt6')
qt6_dep = dependency('qt6', modules: ['Core', 'Gui', 'Widgets'])

project_source_files = [
  'src/mainwindow.cpp',
  'src/aboutdialog.cpp',
  'src/imageprocessor.cpp',
  'src/main.cpp'
]

project_include_files = include_directories('include')

project_dependencies = [
  qt6_dep
]

moc_files = qt6.compile_moc(headers : ['include/mainwindow.h', 'include/aboutdialog.h'],
  include_directories: project_include_files,
  dependencies: qt6_dep
)

compiled_ui_files = qt6.compile_ui(sources: ['ui/mainwindow.ui', 'ui/aboutdialog.ui'])

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

project_target = executable(
  meson.project_name(),
  moc_files,
  project_source_files,
  compiled_ui_files,
  dependencies: project_dependencies,
  include_directories: project_include_files,
  cpp_args : build_args,
)

test('basic', project_target)
