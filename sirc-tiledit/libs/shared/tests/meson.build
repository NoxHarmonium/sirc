test_dependencies = [dependency('catch2-with-main', version : '>=3.0.0')]

conf = configuration_data()
conf.set_quoted('TEST_ROOT', meson.source_root() / 'libs/shared/tests')
configure_file(
    output : 'testconfig.h',
    configuration : conf
)

shared_test_source_files = [
    './utils.cpp',
]

# Tests

test(
    'SircImage',
    executable(
        'sircimage',
        'sircimage.test.cpp',
        shared_test_source_files,
        dependencies : test_dependencies,
        include_directories : shared_library_include_files,
        link_with : shared_lib,
    ),
)

test(
    'MedianCutQuantizer',
    executable(
        'mediancutquantizer',
        'mediancutquantizer.test.cpp',
        shared_test_source_files,
        dependencies : test_dependencies,
        include_directories : shared_library_include_files,
        link_with : shared_lib,
    ),
)

test(
    'Integration',
    executable(
        'integration',
        'integration.test.cpp',
        shared_test_source_files,
        dependencies : test_dependencies,
        include_directories : shared_library_include_files,
        link_with : shared_lib,
    ),
)