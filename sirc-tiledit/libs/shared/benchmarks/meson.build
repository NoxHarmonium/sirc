conf = configuration_data()
conf.set_quoted('BENCHMARK_ROOT', meson.global_source_root() / 'libs/shared/benchmarks')
configure_file(
    output : 'testconfig.h',
    configuration : conf
)

nanobench_lib = library(
    'nanobench',
    ['./nanobench/nanobench.cpp'],
    version : '4.3.11',
    cpp_args : cpp_args
)


benchmark(
    'AllBenchmarks',
    executable(
        'benchmarks',
        ['benchmarks.cpp'],
        include_directories : shared_library_include_files,
        link_with : [nanobench_lib, shared_lib],
        cpp_args : cpp_args
    ),
)

