# AccessKit cross-platform SDL example

This example demonstrates how to make use of the C bindings to create cross-platform applications.

## Building

The process will vary based on your operating system.

### On Windows:

First download a copy of SDL2 and extract it.

```bash
cmake -S . -B build -DACCESSKIT_DIR="../.." -DSDL2_ROOT="<PATH_TO_SDL2_INSTALLATION>/cmake"
cmake --build build --config Release
```

You will then need to copy `SDL2.dll` into the `build/Release` folder.

### On Linux

Make sure to install SDL2 and its development package.

```bash
cmake -S . -B build -DACCESSKIT_DIR="../.." -DCMAKE_BUILD_TYPE=Release
cmake --build build
```
