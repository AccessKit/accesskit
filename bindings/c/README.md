# AccessKit C bindings

These are the bindings to use AccessKit from other languages through FFI such as C.

Inside the `bindings/c` directory, some of the available make recipes include:

- `make include/accesskit.h`: generate the header files
- `make ../../target/release/libaccesskit.a`: build static and dynamic libraries on Linux
- `make clean`: to remove generated artifacts
