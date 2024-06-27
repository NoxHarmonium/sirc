project('sirc-tiledit',
        'cpp',
        version : '1.0.0',
        license : 'AGPL-3.0-only',
        default_options : ['cpp_std=c++20', 'warning_level=3', 'werror=true']
)
add_global_arguments(['-DPROJECT_NAME=' + meson.project_name(),
                      '-DPROJECT_VERSION=' + meson.project_version(),
                      '-Wshadow',
                     ], language : ['cpp'])

qt6 = import('qt6')
qt6_dep = dependency('qt6', modules : ['Core', 'Gui', 'Widgets'])

gui_project_dependencies = [qt6_dep]
gui_source_files = [
    'libs/gui/src/mainwindow.cpp',
    'libs/gui/src/aboutdialog.cpp',
    'libs/gui/src/main.cpp',
    'libs/gui/src/pixmapadapter.cpp',
]

shared_library_source_files = ['libs/shared/src/sircimage.cpp', 'libs/shared/src/mediancutquantizer.cpp', 'libs/shared/src/imageloader.cpp', 'libs/shared/src/rgbaadapter.cpp', ]
shared_library_include_files = include_directories('libs/shared/include')
shared_library_dependencies = [dependency('libpng')]

moc_files = qt6.compile_moc(
    headers : ['libs/gui/src/mainwindow.hpp', 'libs/gui/src/aboutdialog.hpp'],
    dependencies : qt6_dep,
)

compiled_ui_files = qt6.compile_ui(sources : ['libs/gui/ui/mainwindow.ui', 'libs/gui/ui/aboutdialog.ui'])

shared_lib = library(
    meson.project_name() + '-shared',
    shared_library_source_files,
    version : meson.project_version(),
    dependencies : shared_library_dependencies,
    include_directories : shared_library_include_files,
)

executable(
    meson.project_name() + '-gui',
    moc_files,
    gui_source_files,
    compiled_ui_files,
    dependencies : gui_project_dependencies,
    include_directories : shared_library_include_files,
    link_with : shared_lib,
)

subdir('libs/shared/tests')
