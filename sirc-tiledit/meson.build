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

env = environment()
env.set('DYLD_LIBRARY_PATH', '../third-party/libsirc/lib/')
env.set('LD_LIBRARY_PATH', '../third-party/libsirc/lib/')

fs = import('fs')

# TODO: disable error on hash warnings until meson releases https://github.com/mesonbuild/meson/issues/14440
cc = meson.get_compiler ('cpp')
if (cc.has_argument ('-Wno-error=#warnings'))
    # On GCC the argument is -Wno-error=cpp instead, but we don't need it in that case anyway, its a Mac/clang issue
    add_global_arguments(['-Wno-error=#warnings'], language : ['cpp'])
endif

qt6 = import('qt6')
qt6_dep = dependency('qt6', modules : ['Core', 'Gui', 'Widgets'])

# TODO: If libsirc isn't found, should it disable any related features but still build?
libsirc_inc = []
if fs.is_dir('third-party/libsirc/include')
    libsirc_inc += include_directories('third-party/libsirc/include')
endif
if fs.is_dir('/usr/local/include/libsirc')
    libsirc_inc += include_directories('/usr/local/include/libsirc')
endif
libsirc_dep = declare_dependency(dependencies : cc.find_library(
                                     'sirc',
                                     dirs : [
                                         join_paths(meson.global_source_root(), 'third-party/libsirc/lib'),
                                         join_paths(meson.global_source_root(), 'third-party/libsirc/lib/x86_64-linux-gnu'),
                                         '/usr/local/lib/x86_64-linux-gnu', # Debian/Ubuntu multiarch
                                         '/usr/local/lib', # Standard Unix
                                         '/usr/lib/x86_64-linux-gnu', # System-wide multiarch
                                         '/usr/lib'                           # System-wide
                                     ],
                                     required : false
                                 ),
                                 include_directories : libsirc_inc)


gui_project_dependencies = [qt6_dep, libsirc_dep]
gui_source_files = [
    'libs/gui/src/mainwindow.cpp',
    'libs/gui/src/aboutdialog.cpp',
    'libs/gui/src/main.cpp',
    'libs/gui/src/pixmapadapter.cpp',
]

shared_library_source_files = [
    'libs/shared/src/imageloader.cpp',
    'libs/shared/src/imagemerger.cpp',
    'libs/shared/src/imageexporter.cpp',
    'libs/shared/src/mediancutquantizer.cpp',
    'libs/shared/src/miscadapter.cpp',
    'libs/shared/src/rgbaadapter.cpp',
]
shared_library_include_files = [include_directories('libs/shared/include'), libsirc_inc]
shared_library_dependencies = [dependency('libpng'), libsirc_dep]

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
subdir('libs/shared/benchmarks')
