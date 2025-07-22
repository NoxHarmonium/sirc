# libsirc - C++ Compatible Bindings for the sirc-vm Toolchain

## Summary

This crate exists to expose sirc-vm functionality to external C/C++ projects.

Why? The sirc-tiledit application is a QT C++ application and needs to be able to
do things like export assembly code. So that I don't have to write duplicate
toolchain code in that application, I'm exposing it from this library.

Why didn't I write that application in Rust? To learn about Rust/C++ interop of course
(not for any practical reason).

## Artefacts

This crate, when built with c-build, will generate the following artefacts:

- Dynamic Libraries
- Static Libraries
- Header Files
- pkg-config .pc file

Which is everything we need to package it up and distribute it.

## Versioning

- The major version will be incremented any time there is a breaking change to existing interfaces.
- The minor version will be incremented when there is a new feature.
- The patch version will be incremented when there is a bugfix.

## Usage

### Getting Started

You need to install c-build to use this crate.

```shell
$ cargo install cargo-c@0.10.14+cargo-0.89.0
```

### Building

To simply build everything to make sure everything is working, use the cbuild command.

```shell
$ cargo cbuild
```

This will build the crate in debug mode.

Everything will be in the target folder under the name of your system architecture.
E.g. on my Mac its at `../target/aarch64-apple-darwin/debug/`.

```shell
$ ls -1  ../target/aarch64-apple-darwin/debug/                                                                                      Tue 22 Jul 08:32:59 2025
build/
cargo-c-libsirc.cache
deps/
examples/
include/
incremental/
libsirc-uninstalled.pc
libsirc.a
libsirc.d
libsirc.dylib*
libsirc.h
libsirc.pc
```

### Exporting

#### For sirc-tiledit development

So we don't need to install the libraries into the system directories during development, you can export the files
directly to the sirc-tiledit project.

```shell
$ cargo cinstall --meson-paths --prefix=/ --destdir=../../sirc-tiledit/third-party/libsirc
```

Then when you run `meson setup` it should find the include/library files from that directory.

#### For packaging/system folders

To export the files in the correct file structure ready for packaging, use the cinstall command.

```shell
$ cargo cinstall --meson-paths --prefix=/usr/local/ --destdir=/tmp/blah
```

This will build the crate in release mode.

Everything will be put into the specified destination dir under the correct prefixes and correctly versioned.

Depending on your platform, if you set prefix correctly, setting destdir to `/` will install the libraries in your
system folders so you can link to them without any extra configuration.

```shell
$ find /tmp/blah/                                                                                                                   Tue 22 Jul 08:38:26 2025
/tmp/blah/
/tmp/blah/usr
/tmp/blah/usr/local
/tmp/blah/usr/local/include
/tmp/blah/usr/local/include/libsirc
/tmp/blah/usr/local/include/libsirc/libsirc.h
/tmp/blah/usr/local/lib
/tmp/blah/usr/local/lib/libsirc.1.0.0.dylib
/tmp/blah/usr/local/lib/pkgconfig
/tmp/blah/usr/local/lib/pkgconfig/libsirc.pc
/tmp/blah/usr/local/lib/libsirc.1.dylib
/tmp/blah/usr/local/lib/libsirc.a
/tmp/blah/usr/local/lib/libsirc.dylib
```