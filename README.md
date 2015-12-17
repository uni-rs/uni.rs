# Uni.rs [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE) [![Build Status](https://travis-ci.org/uni-rs/uni.rs.svg)](https://travis-ci.org/uni-rs/uni.rs)

Uni.rs is an experimental [unikernel](https://en.wikipedia.org/wiki/Unikernel)
written in rust.

The project is a work in progress and therefore is highly unstable and subject
to design, API, ... changes.

## Supported plaforms

| Platform      | Architecture  |
| ------------- | ------------- |
| Xen           | x86, x86_64   |

## Usage

### Building examples

You can find several examples in this source tree inside the directory named
`examples`. Uni.rs is using cargo to build the libraries and examples. At the
moment examples do not have separated Cargo.toml files. Indeed they can be
built using the root Cargo.toml file. Here is the list of example name
associated with source files in the examples directory.

- hello => examples/hello/main.rs
- thread => examples/thread/main.rs
- queue => examples/thread/queue.rs

In order to build the example you might need to add the feature `with-core`.
Since we use a custom target, the core library probably does not exist in your
environment. As a convenience this feature will build the necessary
dependencies from the rust sources. Here is the command that one might use to
build the `hello` example.

```
$ cargo build --target x86_64-unknown-uni.json --release --features with-core --example hello
```

### Using in your own projects

Since Uni.rs is using cargo as its build system you simply need to properly
setup your Cargo.toml to use this repository. You also *MUST* build your code
using the custom targets that are available in this repository.
