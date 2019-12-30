# libssh-sys
Rust crate that provides FFI bindings to [libssh](https://www.libssh.org).

## Dependencies
Bindings are generated at build time using
[bindgen](https://github.com/rust-lang/rust-bindgen). See its
[Requirements page](https://rust-lang.github.io/rust-bindgen/requirements.html).
`libssh` must be present on your system during runtime, it is dynamically linked.
This addresses the [warnings](https://api.libssh.org/stable/libssh_linking.html)
against static linking and its license implications.
In order to build this crate `libssh` header files must also be available.

On Ubuntu all dependencies can be installed by running
```
apt install libssh-dev llvm-dev clang
```
See other options on [libssh download page](https://www.libssh.org/get-it/).

## Building
```
cargo build
```

## Usage
See [smoke test](tests/smoke_test.rs) where simple ssh server and client are
created. The goal of the test is to show bindings work correctly, it is not
a recommended way of API usage. More examples in C/C++ can be found
[here](https://git.libssh.org/projects/libssh.git/tree/examples).

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE-2.0) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Note that `libssh` is licensed under LGPLv2.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

