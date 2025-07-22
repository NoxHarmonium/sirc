# SIRC Tiledit

A QT based GUI for manipulating tile data.

## Building

I've been using Qt Creator to work on this project
but it should be able to work in other IDEs that
support clangd (although you wouldn't get the
UI editor)

```
# or ./setup-macos.sh if you want to use homebrew llvm
$ meson setup build

$ cd build
$ meson compile
$ meson test

# You need to compile before these steps, otherwise the qt generated headers won't be there
$ ninja clang-tidy
$ ninja clang-format-check
```

If you have the libsirc libraries at third-party/libsirc/lib
(recommended during development), you'll need to make sure you're
setting the library search path at runtime.

E.g. on MacOS

```shell
DYLD_LIBRARY_PATH=./third-party/libsirc/lib/ ./build/sirc-tiledit-gui
```

or Linux

```shell
LD_LIBRARY_PATH=./third-party/libsirc/lib/ ./build/sirc-tiledit-gui
```

If you're not actively working on both libsirc (sirc-vm) and this project
at the same time, you can probably install libsirc into your system library
directories to avoid having to do this.

# Roadmap

- [x] Get a boilerplate QT app running
- [x] Get some quantization working to reduce palette size for tile data
- [ ] Export tile data as assembly files for import into projects
- [ ] Manage tilemap data
- [ ] Manage sprite data
