# Builder Dockerfile

It was annoying to get a consistent environment to build the project in since
it is currently being developed on Fedora Linux and MacOS and built on Ubuntu.

This should be the source of truth for the build environment and used as a reference for versions etc.

# Usage

## Building it

From this directory:

```shell
docker build -t sirc:tiledit-builder -f ./builder.Dockerfile .
```

## Using it

From this directory:

```shell
podman run --rm -v"$(pwd)/..":/project:z sirc:tiledit-builder meson compile
podman run --rm -v"$(pwd)/..":/project:z sirc:tiledit-builder meson test


```


