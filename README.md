# Uni.rs [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

Uni.rs is an experimental [unikernel](https://en.wikipedia.org/wiki/Unikernel)
written in rust.

The project is a work in progress and therefore is highly instable and subject
to design, API, ... changes.

## Supported plaforms

| Platform      | Architecture  |
| ------------- | ------------- |
| Xen           | x86, x86_64   |

## Build Instructions

The build system will ease you the task of building your applications using the
Uni.rs unikernel. In order to do so, a target called `bin` is responsible to
build all Uni.rs libraries and link your code with it.

This target can be customized by 2 variables:
- BIN_PATH: path to your main .rs file
- BIN_OUTPUT: path to the output file

For example if you want to generate the hello binary from the example
directory (examples/hello), you would do something like this:
`BIN_PATH=./examples/hello/main.rs BIN_OUTPUT=hello make bin`
This will generate an hello binary in the current directory from the
rust file named `examples/hello/main.rs`.

## Dependencies

- A multirust installation
