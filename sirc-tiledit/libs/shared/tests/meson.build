conf = configuration_data()
conf.set_quoted('TEST_ROOT', meson.source_root() / 'libs/shared/tests')
configure_file(
    output : 'testconfig.h',
    configuration : conf
)

shared_test_source_files = [
    './utils.cpp',
    './catch2/catch_amalgamated.cpp'
]

# Tests

test(
    'MedianCutQuantizer',
    executable(
        'mediancutquantizer',
        'mediancutquantizer.test.cpp',
        shared_test_source_files,
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
        include_directories : shared_library_include_files,
        link_with : shared_lib,
    ),
)