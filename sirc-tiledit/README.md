# SIRC Tiledit

A QT based GUI for manipulating tile data.

## Building

I've been using Qt Creator to work on this project
but it should be able to work in other IDEs that
support clangd (although you wouldn't get the 
UI editor)

```
$ meson setup build
$ cd build
$ meson compile
$ meson test
```

# Roadmap

- [x] Get a boilerplate QT app running
- [x] Get some quantization working to reduce palette size for tile data
- [ ] Export tile data as assembly files for import into projects
- [ ] Manage tilemap data
- [ ] Manage sprite data