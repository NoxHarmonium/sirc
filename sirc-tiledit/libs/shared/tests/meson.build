conf = configuration_data()
conf.set_quoted('TEST_ROOT', meson.global_source_root() / 'libs/shared/tests')
configure_file(
    output : 'testconfig.h',
    configuration : conf
)

shared_test_source_files = [
    './test_utils.cpp',
    './catch2/catch_amalgamated.cpp'
]
gui_project_dependencies = [libsirc_dep]

test_env = ['DYLD_LIBRARY_PATH=' + join_paths(meson.global_source_root(), 'third-party/libsirc/lib/'),
            'LD_LIBRARY_PATH=' + join_paths(meson.global_source_root(), 'third-party/libsirc/lib/')]

# Tests

test(
    'ImageExporter',
    executable(
        'test_imageexporter',
        'imageexporter.test.cpp',
        shared_test_source_files,
        include_directories : shared_library_include_files,
        link_with : shared_lib,
        cpp_args : cpp_args

    ),
    env : test_env
)


test(
    'ImageMerger',
    executable(
        'test_imagemerger',
        'imagemerger.test.cpp',
        shared_test_source_files,
        include_directories : shared_library_include_files,
        link_with : shared_lib,
        cpp_args : cpp_args

    ),
    env : test_env
)

test(
    'MedianCutQuantizer',
    executable(
        'test_mediancutquantizer',
        'mediancutquantizer.test.cpp',
        shared_test_source_files,
        include_directories : shared_library_include_files,
        link_with : shared_lib,
        cpp_args : cpp_args

    ),
    env : test_env
)

test(
    'Utils',
    executable(
        'test_utils',
        'utils.test.cpp',
        shared_test_source_files,
        include_directories : shared_library_include_files,
        link_with : shared_lib,
        cpp_args : cpp_args

    ),
    env : test_env
)

test(
    'Integration',
    executable(
        'test_integration',
        'integration.test.cpp',
        shared_test_source_files,
        include_directories : shared_library_include_files,
        link_with : shared_lib,
        cpp_args : cpp_args

    ),
    env : test_env
)